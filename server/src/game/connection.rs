use std::{future::pending, sync::Arc, time::Duration};

use axum::extract::ws::{Message, WebSocket};
use grenadiers_core::protocol::{
    Counter, ErrorCode, GameClientMessage, GameServerMessage, RoomId, parse_client_message,
};
use tokio::time::{Instant, MissedTickBehavior, interval_at, sleep_until};

use super::{
    limits::{InboundLimit, SessionDeadline, SessionDeadlines},
    rooms::{Admission, BindingKey, GameRooms, RoomError, RoomNotice},
    sessions::ConnectionId,
    socket::{close, command_sequence, fail_and_close, send_error, send_game, send_raw},
};

const PING_INTERVAL: Duration = Duration::from_secs(10);

struct Membership {
    admission: Admission,
    binding: BindingKey,
}

struct ConnectionState {
    connection_id: ConnectionId,
    generation: u64,
    membership: Option<Membership>,
    highwater: Option<Counter>,
    deadlines: SessionDeadlines,
}

pub(super) async fn authenticated_loop(
    socket: &mut WebSocket,
    inbound: &mut InboundLimit,
    connection_id: ConnectionId,
    rooms: Arc<GameRooms>,
) {
    let started = Instant::now();
    let mut state = ConnectionState {
        connection_id,
        generation: 0,
        membership: None,
        highwater: None,
        deadlines: SessionDeadlines::new(started),
    };
    let mut pings = interval_at(started + PING_INTERVAL, PING_INTERVAL);
    pings.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        enum Event {
            Incoming(Option<Message>),
            Notice(Result<(), tokio::sync::watch::error::RecvError>),
            Ping,
            Inactive,
            Expired,
        }
        let is_member = state.membership.is_some();
        let event = tokio::select! {
            incoming = socket.recv() => Event::Incoming(incoming.and_then(Result::ok)),
            notice = notice_changed(&mut state.membership) => Event::Notice(notice),
            _ = pings.tick() => Event::Ping,
            () = sleep_until(state.deadlines.inactivity_at()) => Event::Inactive,
            () = unseated_expiry(&state.deadlines, is_member) => Event::Expired,
        };
        let now = Instant::now();
        let status = if state.membership.is_some() {
            state.deadlines.classify_member(now)
        } else {
            state.deadlines.classify(now)
        };
        match status {
            SessionDeadline::Expired => {
                close(socket, 1000, "Session expired").await;
                return;
            }
            SessionDeadline::Inactive => {
                close(socket, 1000, "Connection inactive").await;
                return;
            }
            SessionDeadline::Active => {}
        }
        match event {
            Event::Ping => {
                if send_raw(socket, Message::Ping(Vec::new().into()))
                    .await
                    .is_err()
                {
                    return;
                }
            }
            Event::Inactive => {
                close(socket, 1000, "Connection inactive").await;
                return;
            }
            Event::Expired => {
                close(socket, 1000, "Session expired").await;
                return;
            }
            Event::Notice(Ok(())) => {
                let Some(member) = state.membership.as_mut() else {
                    continue;
                };
                let notice = member.admission.notices.borrow_and_update().clone();
                match notice {
                    RoomNotice::View(room) => {
                        if send_game(
                            socket,
                            &GameServerMessage::RoomState {
                                room,
                                you_seat: member.admission.seat,
                            },
                        )
                        .await
                        .is_err()
                        {
                            return;
                        }
                    }
                    RoomNotice::Expired => {
                        let _ = send_error(socket, ErrorCode::RoomNotFound, None).await;
                        close(socket, 1000, "Room expired").await;
                        return;
                    }
                }
            }
            Event::Notice(Err(_)) => {
                if let Some(member) = state.membership.as_ref() {
                    rooms.cleanup(&member.admission.room_id, &member.admission.view.room_code);
                }
                close(socket, 1000, "Room unavailable").await;
                return;
            }
            Event::Incoming(None | Some(Message::Close(_))) => return,
            Event::Incoming(Some(message)) => {
                state.deadlines.record_inbound(now);
                if !inbound.take(now) {
                    fail_and_close(socket, ErrorCode::RateLimited, None).await;
                    return;
                }
                if !handle_message(socket, message, &rooms, &mut state).await {
                    return;
                }
            }
        }
    }
}

async fn handle_message(
    socket: &mut WebSocket,
    message: Message,
    rooms: &Arc<GameRooms>,
    state: &mut ConnectionState,
) -> bool {
    match message {
        Message::Ping(data) => send_raw(socket, Message::Pong(data)).await.is_ok(),
        Message::Pong(_) => true,
        Message::Binary(_) => {
            fail_and_close(socket, ErrorCode::InvalidMessage, None).await;
            false
        }
        Message::Close(_) => false,
        Message::Text(text) => {
            let Ok(message) = parse_client_message(&text) else {
                fail_and_close(socket, ErrorCode::InvalidMessage, None).await;
                return false;
            };
            if matches!(message, GameClientMessage::Authenticate { .. }) {
                fail_and_close(socket, ErrorCode::AlreadyAuthenticated, None).await;
                return false;
            }
            let sequence = command_sequence(&message).expect("command has sequence");
            if state.highwater.is_some_and(|seen| sequence <= seen) {
                return send_error(socket, ErrorCode::StaleSequence, Some(sequence))
                    .await
                    .is_ok();
            }
            state.highwater = Some(sequence);
            dispatch(socket, message, sequence, rooms, state).await
        }
    }
}

async fn dispatch(
    socket: &mut WebSocket,
    message: GameClientMessage,
    sequence: Counter,
    rooms: &Arc<GameRooms>,
    state: &mut ConnectionState,
) -> bool {
    match message {
        GameClientMessage::CreateRoom { .. } => {
            if state.membership.is_some() {
                return reply_error(socket, ErrorCode::AlreadyInRoom, sequence).await;
            }
            let Some(next_generation) = state.generation.checked_add(1) else {
                return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
            };
            let binding = BindingKey {
                connection_id: state.connection_id,
                generation: next_generation,
            };
            let result = rooms.create(binding, std::time::Instant::now()).await;
            admit(socket, result, binding, state, sequence).await
        }
        GameClientMessage::JoinRoom { room_code, .. } => {
            if state.membership.is_some() {
                return reply_error(socket, ErrorCode::AlreadyInRoom, sequence).await;
            }
            let Some(next_generation) = state.generation.checked_add(1) else {
                return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
            };
            let binding = BindingKey {
                connection_id: state.connection_id,
                generation: next_generation,
            };
            let result = rooms
                .join(&room_code, binding, std::time::Instant::now())
                .await;
            admit(socket, result, binding, state, sequence).await
        }
        GameClientMessage::LeaveRoom {
            room_id, round_id, ..
        } => {
            leave(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                state,
                sequence,
            )
            .await
        }
        GameClientMessage::Ready {
            room_id, round_id, ..
        }
        | GameClientMessage::Movement {
            room_id, round_id, ..
        }
        | GameClientMessage::PlaceBomb {
            room_id, round_id, ..
        } => {
            placeholder(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                state,
                sequence,
            )
            .await
        }
        GameClientMessage::Authenticate { .. } => unreachable!(),
    }
}

async fn admit(
    socket: &mut WebSocket,
    result: Result<Admission, RoomError>,
    binding: BindingKey,
    state: &mut ConnectionState,
    sequence: Counter,
) -> bool {
    match result {
        Ok(admission) => {
            let message = GameServerMessage::RoomState {
                room: admission.view.clone(),
                you_seat: admission.seat,
            };
            if send_game(socket, &message).await.is_err() {
                return false;
            }
            state.generation = binding.generation;
            state.membership = Some(Membership { admission, binding });
            true
        }
        Err(error) => room_error(socket, error, sequence).await,
    }
}

async fn leave(
    socket: &mut WebSocket,
    rooms: &GameRooms,
    room_id: RoomId,
    round_id: u64,
    state: &mut ConnectionState,
    sequence: Counter,
) -> bool {
    let Some(member) = state.membership.as_ref() else {
        return reply_error(socket, ErrorCode::NotInRoom, sequence).await;
    };
    if member.admission.room_id != room_id || member.admission.round_id != round_id {
        return reply_error(socket, ErrorCode::StaleRoom, sequence).await;
    }
    let Some(next_generation) = state.generation.checked_add(1) else {
        return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
    };
    match rooms.leave(&member.admission, member.binding).await {
        Ok(()) => {
            let mut member = state.membership.take().expect("membership exists");
            member.admission.guard.disarm();
            state.generation = next_generation;
            state.deadlines.restart_unseated(Instant::now());
            send_game(socket, &GameServerMessage::LeftRoom { room_id, sequence })
                .await
                .is_ok()
        }
        Err(error) => room_error(socket, error, sequence).await,
    }
}

async fn placeholder(
    socket: &mut WebSocket,
    rooms: &GameRooms,
    room_id: RoomId,
    round_id: u64,
    state: &ConnectionState,
    sequence: Counter,
) -> bool {
    let Some(member) = state.membership.as_ref() else {
        return reply_error(socket, ErrorCode::NotInRoom, sequence).await;
    };
    match rooms
        .validate(&member.admission, member.binding, room_id, round_id)
        .await
    {
        Ok(()) => reply_error(socket, ErrorCode::InvalidPhase, sequence).await,
        Err(error) => room_error(socket, error, sequence).await,
    }
}

async fn room_error(socket: &mut WebSocket, error: RoomError, sequence: Counter) -> bool {
    let (code, close_after) = match error {
        RoomError::RateLimited => (ErrorCode::RateLimited, true),
        RoomError::CapacityReached => (ErrorCode::CapacityReached, false),
        RoomError::RoomNotFound => (ErrorCode::RoomNotFound, false),
        RoomError::RoomFull => (ErrorCode::RoomFull, false),
        RoomError::StaleRoom => (ErrorCode::StaleRoom, false),
    };
    if close_after {
        fail_and_close(socket, code, Some(sequence)).await;
        false
    } else {
        reply_error(socket, code, sequence).await
    }
}

async fn reply_error(socket: &mut WebSocket, code: ErrorCode, sequence: Counter) -> bool {
    send_error(socket, code, Some(sequence)).await.is_ok()
}

async fn notice_changed(
    membership: &mut Option<Membership>,
) -> Result<(), tokio::sync::watch::error::RecvError> {
    match membership {
        Some(member) => member.admission.notices.changed().await,
        None => pending().await,
    }
}

async fn unseated_expiry(deadlines: &SessionDeadlines, is_member: bool) {
    if is_member {
        pending().await
    } else {
        sleep_until(deadlines.expires_at()).await
    }
}
