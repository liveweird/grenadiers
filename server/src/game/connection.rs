use std::{future::pending, sync::Arc, time::Duration};

use axum::extract::ws::{Message, WebSocket};
use grenadiers_core::protocol::Counter;
use grenadiers_core::protocol::v20::{ErrorCode, GameServerMessage, parse_client_message};
use tokio::time::{Instant, MissedTickBehavior, interval_at, sleep_until, timeout};

use super::{
    command::InboundMessage,
    limits::{InboundLimit, SessionDeadline, SessionDeadlines},
    rooms::{
        BindingKey, ClosingReason, GameRooms, LiveAdmission as Admission, LiveDelivery,
        LiveMemberEvent as MemberEvent,
    },
    sessions::ConnectionId,
    socket::{close, fail_and_close, send_error, send_game, send_raw},
};
use crate::{
    admission::{DenialTelemetry, GameplayAdmission},
    peer_identity::PeerKey,
};

const PING_INTERVAL: Duration = Duration::from_secs(10);
const FINAL_DELIVERY_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) struct Membership {
    pub(super) admission: Admission,
    pub(super) binding: BindingKey,
    pub(super) replacement_closed: bool,
}

pub(super) struct ConnectionState {
    pub(super) connection_id: ConnectionId,
    pub(super) generation: u64,
    pub(super) membership: Option<Membership>,
    pub(super) highwater: Option<Counter>,
    pub(super) deadlines: SessionDeadlines,
    pub(super) peer_key: PeerKey,
    pub(super) peer_admission: GameplayAdmission,
    pub(super) denial_telemetry: Arc<DenialTelemetry>,
}

pub(super) async fn authenticated_loop(
    socket: &mut WebSocket,
    inbound: &mut InboundLimit,
    connection_id: ConnectionId,
    peer_key: PeerKey,
    rooms: Arc<GameRooms>,
    peer_admission: GameplayAdmission,
    denial_telemetry: Arc<DenialTelemetry>,
) {
    let started = Instant::now();
    let mut state = ConnectionState {
        connection_id,
        generation: 0,
        membership: None,
        highwater: None,
        deadlines: SessionDeadlines::new(started),
        peer_key,
        peer_admission,
        denial_telemetry,
    };
    let mut pings = interval_at(started + PING_INTERVAL, PING_INTERVAL);
    pings.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let mut shutdown = rooms.shutdown();
    loop {
        enum Event {
            Incoming(Option<Message>),
            Member(MemberEvent),
            Ping,
            Inactive,
            Expired,
            Shutdown,
        }
        let is_member = state.membership.is_some();
        let event = tokio::select! {
            incoming = socket.recv() => Event::Incoming(incoming.and_then(Result::ok)),
            member = member_event(&mut state.membership) => Event::Member(member),
            _ = pings.tick() => Event::Ping,
            () = sleep_until(state.deadlines.inactivity_at()) => Event::Inactive,
            () = unseated_expiry(&state.deadlines, is_member) => Event::Expired,
            changed = unseated_shutdown(&mut shutdown, is_member) => {
                let _ = changed;
                Event::Shutdown
            },
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
            Event::Shutdown => {
                close(socket, 1001, "Server shutting down").await;
                return;
            }
            Event::Member(MemberEvent::Notice(Ok(()))) => {
                let Some(member) = state.membership.as_mut() else {
                    continue;
                };
                let Ok(delivery) = member.admission.latest_delivery() else {
                    close(socket, 1011, "Room unavailable").await;
                    return;
                };
                match delivery {
                    LiveDelivery::Publication(frame) => {
                        if send_notice(socket, frame).await.is_err() {
                            return;
                        }
                    }
                    LiveDelivery::Closing { last, reason } => {
                        finish_closing(socket, last, reason).await;
                        return;
                    }
                }
            }
            Event::Member(MemberEvent::Notice(Err(_))) => {
                if let Some(member) = state.membership.as_mut()
                    && let Ok(LiveDelivery::Closing { last, reason }) =
                        member.admission.latest_delivery()
                {
                    finish_closing(socket, last, reason).await;
                    return;
                }
                close(socket, 1000, "Room unavailable").await;
                return;
            }
            Event::Member(MemberEvent::Replaced) => {
                let room_id = state
                    .membership
                    .as_ref()
                    .expect("member event")
                    .admission
                    .room_id()
                    .clone();
                let _ = timeout(FINAL_DELIVERY_TIMEOUT, async {
                    let _ = send_game(socket, &GameServerMessage::SeatReplaced { room_id }).await;
                    close(socket, 4001, "Seat replaced").await;
                })
                .await;
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

async fn send_notice(socket: &mut WebSocket, frame: Arc<str>) -> Result<(), ()> {
    send_raw(socket, Message::Text(frame.to_string().into())).await
}

async fn finish_closing(socket: &mut WebSocket, last: Arc<str>, reason: ClosingReason) {
    let _ = timeout(FINAL_DELIVERY_TIMEOUT, async {
        let _ = send_notice(socket, last).await;
        match reason {
            ClosingReason::Expired => {
                let _ = send_error(socket, ErrorCode::RoomNotFound, None).await;
                close(socket, 1000, "Room expired").await;
            }
            ClosingReason::Retired => close(socket, 1000, "Room retired").await,
            ClosingReason::ServerStopped => close(socket, 1001, "Server shutting down").await,
            ClosingReason::InternalFailure => close(socket, 1011, "Room unavailable").await,
        }
    })
    .await;
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
            match InboundMessage::from(message) {
                InboundMessage::Authenticate { .. } => {
                    fail_and_close(socket, ErrorCode::AlreadyAuthenticated, None).await;
                    false
                }
                InboundMessage::Command(command) => {
                    let sequence = command.sequence();
                    if state.highwater.is_some_and(|seen| sequence <= seen) {
                        return send_error(socket, ErrorCode::StaleSequence, Some(sequence))
                            .await
                            .is_ok();
                    }
                    state.highwater = Some(sequence);
                    super::dispatcher::dispatch(socket, command, rooms, state).await
                }
            }
        }
    }
}

async fn member_event(membership: &mut Option<Membership>) -> MemberEvent {
    match membership {
        Some(member) => {
            member
                .admission
                .next_event(&mut member.replacement_closed)
                .await
        }
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

async fn unseated_shutdown(
    shutdown: &mut tokio::sync::watch::Receiver<bool>,
    is_member: bool,
) -> Result<(), tokio::sync::watch::error::RecvError> {
    if is_member {
        pending().await
    } else if *shutdown.borrow_and_update() {
        Ok(())
    } else {
        shutdown.changed().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::rooms::{GameRooms, RoomNotice};

    #[tokio::test(flavor = "current_thread", start_paused = true)]
    async fn seat_two_membership_routes_marked_normal_retained_and_shutdown_frames() {
        let next = std::sync::atomic::AtomicU8::new(1);
        let rooms = GameRooms::with_random_fill(std::time::Instant::now(), move |bytes| {
            bytes.fill(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
            Ok(())
        });
        let binding = |value| BindingKey {
            connection_id: ConnectionId(value),
            generation: 1,
        };
        let first = rooms
            .create(binding(1), std::time::Instant::now())
            .await
            .unwrap();
        let mut second = rooms
            .join(&first.room.room_code, binding(2), std::time::Instant::now())
            .await
            .unwrap();
        let mut marked = match &*second.notices.borrow() {
            RoomNotice::Publication(notice) => notice.clone(),
            RoomNotice::Closing { .. } => panic!("new room is open"),
        };
        marked.replace_frames_for_test(["seat-one", "seat-two"]);
        let (sender, receiver) =
            tokio::sync::watch::channel(RoomNotice::Publication(marked.clone()));
        second.notices = receiver;
        let mut admitted = Admission::Legacy(second);
        for notice in [
            RoomNotice::Publication(marked.clone()),
            RoomNotice::Closing {
                last: marked.clone(),
                reason: ClosingReason::Expired,
            },
            RoomNotice::Closing {
                last: marked,
                reason: ClosingReason::ServerStopped,
            },
        ] {
            sender.send_replace(notice);
            let frame = match admitted.latest_delivery().unwrap() {
                LiveDelivery::Publication(frame) | LiveDelivery::Closing { last: frame, .. } => {
                    frame
                }
            };
            assert_eq!(&*frame, "seat-two");
        }
        drop(first);
    }
}
