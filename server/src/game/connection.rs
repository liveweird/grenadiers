use std::{future::pending, sync::Arc, time::Duration};

use axum::extract::ws::{Message, WebSocket};
use grenadiers_core::protocol::{
    Counter, ErrorCode, GameClientMessage, GameServerMessage, parse_client_message,
};
use tokio::time::{Instant, MissedTickBehavior, interval_at, sleep_until};

use super::{
    limits::{InboundLimit, SessionDeadline, SessionDeadlines},
    rooms::{Admission, BindingKey, GameRooms, RoomNotice},
    sessions::ConnectionId,
    socket::{close, command_sequence, fail_and_close, send_error, send_game, send_raw},
};

const PING_INTERVAL: Duration = Duration::from_secs(10);

pub(super) struct Membership {
    pub(super) admission: Admission,
    pub(super) binding: BindingKey,
}

pub(super) struct ConnectionState {
    pub(super) connection_id: ConnectionId,
    pub(super) generation: u64,
    pub(super) membership: Option<Membership>,
    pub(super) highwater: Option<Counter>,
    pub(super) deadlines: SessionDeadlines,
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
                    RoomNotice::Snapshot(snapshot) => {
                        if send_raw(socket, Message::Text(snapshot.to_string().into()))
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
                    RoomNotice::Retired => {
                        close(socket, 1000, "Room retired").await;
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
            super::dispatcher::dispatch(socket, message, sequence, rooms, state).await
        }
    }
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
