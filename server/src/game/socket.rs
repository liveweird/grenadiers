use std::time::{Duration, Instant as StdInstant};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{CloseFrame, Message, WebSocket},
    },
    http::{HeaderMap, StatusCode, Uri, header::RETRY_AFTER},
    response::Response,
};
use grenadiers_core::protocol::{
    Counter, ErrorCode, GAMEPLAY_PROTOCOL_VERSION, GameClientMessage, GameServerMessage,
    GameplayVersion, MAX_CLIENT_MESSAGE_BYTES, encode_server_message, parse_client_message,
};
use tokio::{
    sync::OwnedSemaphorePermit,
    time::{Instant, MissedTickBehavior, interval_at, sleep_until, timeout, timeout_at},
};

use super::{
    http::origin_allowed,
    limits::{InboundLimit, SessionDeadline, SessionDeadlines},
    sessions::SessionClaimError,
};
use crate::{AppState, Problem, with_no_store};

const AUTH_TIMEOUT: Duration = Duration::from_secs(5);
const SEND_TIMEOUT: Duration = Duration::from_secs(5);
const PING_INTERVAL: Duration = Duration::from_secs(10);
const MAX_WRITE_BUFFER: usize = 131_072;

pub(crate) async fn game_websocket(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    if uri.query().is_some_and(|query| !query.is_empty()) {
        return with_no_store(
            Problem::new(
                StatusCode::BAD_REQUEST,
                "Bad Request",
                "Query parameters are not supported",
            )
            .response(),
        );
    }
    if !origin_allowed(&headers, &state) {
        return with_no_store(
            Problem::new(StatusCode::FORBIDDEN, "Forbidden", "Origin is not allowed").response(),
        );
    }
    if !state.game_sessions.allow_upgrade(StdInstant::now()) {
        let mut response = Problem::new(
            StatusCode::TOO_MANY_REQUESTS,
            "Too Many Requests",
            "Connection rate limit reached",
        )
        .response();
        response
            .headers_mut()
            .insert(RETRY_AFTER, "1".parse().expect("valid retry after"));
        return with_no_store(response);
    }
    let Ok(permit) = state.connections.clone().try_acquire_owned() else {
        return with_no_store(
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                "WebSocket connection limit reached",
            )
            .response(),
        );
    };
    let sessions = state.game_sessions;
    upgrade
        .max_frame_size(MAX_CLIENT_MESSAGE_BYTES)
        .max_message_size(MAX_CLIENT_MESSAGE_BYTES)
        .write_buffer_size(0)
        .max_write_buffer_size(MAX_WRITE_BUFFER)
        .on_upgrade(move |socket| game_session(socket, permit, sessions))
}

async fn game_session(
    mut socket: WebSocket,
    _permit: OwnedSemaphorePermit,
    sessions: std::sync::Arc<super::sessions::GameSessions>,
) {
    let mut inbound = InboundLimit::new(Instant::now());
    let deadline = Instant::now() + AUTH_TIMEOUT;
    let lease = loop {
        let incoming = match timeout_at(deadline, socket.recv()).await {
            Ok(Some(Ok(message))) => message,
            Ok(Some(Err(_)) | None) => return,
            Err(_) => {
                fail_and_close(&mut socket, ErrorCode::NotAuthenticated, None).await;
                return;
            }
        };
        if Instant::now() >= deadline {
            fail_and_close(&mut socket, ErrorCode::NotAuthenticated, None).await;
            return;
        }
        if !inbound.take(Instant::now()) {
            fail_and_close(&mut socket, ErrorCode::RateLimited, None).await;
            return;
        }
        match incoming {
            Message::Ping(data) => {
                if send_raw(&mut socket, Message::Pong(data)).await.is_err() {
                    return;
                }
            }
            Message::Pong(_) => {}
            Message::Close(_) => return,
            Message::Binary(_) => {
                fail_and_close(&mut socket, ErrorCode::InvalidMessage, None).await;
                return;
            }
            Message::Text(text) => {
                let Ok(message) = parse_client_message(&text) else {
                    fail_and_close(&mut socket, ErrorCode::InvalidMessage, None).await;
                    return;
                };
                let GameClientMessage::Authenticate {
                    protocol_version,
                    token,
                } = message
                else {
                    fail_and_close(
                        &mut socket,
                        ErrorCode::NotAuthenticated,
                        command_sequence(&message),
                    )
                    .await;
                    return;
                };
                if protocol_version.get() != GAMEPLAY_PROTOCOL_VERSION {
                    let _ = send_game(
                        &mut socket,
                        &GameServerMessage::IncompatibleProtocol {
                            supported_version: GameplayVersion,
                        },
                    )
                    .await;
                    close(&mut socket, 1008, "Incompatible protocol").await;
                    return;
                }
                match sessions.claim(&token, StdInstant::now()) {
                    Ok(lease) => {
                        if send_game(
                            &mut socket,
                            &GameServerMessage::Authenticated {
                                protocol_version: GameplayVersion,
                            },
                        )
                        .await
                        .is_err()
                        {
                            return;
                        }
                        break lease;
                    }
                    Err(error) => {
                        let code = match error {
                            SessionClaimError::AuthenticationFailed => {
                                ErrorCode::AuthenticationFailed
                            }
                            SessionClaimError::ConnectionInUse => ErrorCode::ConnectionInUse,
                            SessionClaimError::CapacityReached => ErrorCode::CapacityReached,
                        };
                        fail_and_close(&mut socket, code, None).await;
                        return;
                    }
                }
            }
        }
    };
    authenticated_loop(&mut socket, &mut inbound).await;
    drop(lease);
}

async fn authenticated_loop(socket: &mut WebSocket, inbound: &mut InboundLimit) {
    let started = Instant::now();
    let mut deadlines = SessionDeadlines::new(started);
    let mut highwater: Option<Counter> = None;
    let mut pings = interval_at(started + PING_INTERVAL, PING_INTERVAL);
    pings.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        enum Event {
            Incoming(Option<Message>),
            Ping,
            Inactive,
            Expired,
        }
        let event = tokio::select! {
            incoming = socket.recv() => Event::Incoming(incoming.and_then(Result::ok)),
            _ = pings.tick() => Event::Ping,
            () = sleep_until(deadlines.inactivity_at()) => Event::Inactive,
            () = sleep_until(deadlines.expires_at()) => Event::Expired,
        };
        let now = Instant::now();
        match deadlines.classify(now) {
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
            Event::Incoming(None | Some(Message::Close(_))) => return,
            Event::Incoming(Some(message)) => {
                deadlines.record_inbound(now);
                if !inbound.take(now) {
                    fail_and_close(socket, ErrorCode::RateLimited, None).await;
                    return;
                }
                match message {
                    Message::Ping(data) => {
                        if send_raw(socket, Message::Pong(data)).await.is_err() {
                            return;
                        }
                    }
                    Message::Pong(_) => {}
                    Message::Binary(_) => {
                        fail_and_close(socket, ErrorCode::InvalidMessage, None).await;
                        return;
                    }
                    Message::Text(text) => {
                        let Ok(message) = parse_client_message(&text) else {
                            fail_and_close(socket, ErrorCode::InvalidMessage, None).await;
                            return;
                        };
                        if matches!(message, GameClientMessage::Authenticate { .. }) {
                            fail_and_close(socket, ErrorCode::AlreadyAuthenticated, None).await;
                            return;
                        }
                        let sequence = command_sequence(&message).expect("command has sequence");
                        if highwater.is_some_and(|seen| sequence <= seen) {
                            if send_error(socket, ErrorCode::StaleSequence, Some(sequence))
                                .await
                                .is_err()
                            {
                                return;
                            }
                            continue;
                        }
                        highwater = Some(sequence);
                        let code = match message {
                            GameClientMessage::CreateRoom { .. }
                            | GameClientMessage::JoinRoom { .. } => ErrorCode::InvalidPhase,
                            _ => ErrorCode::NotInRoom,
                        };
                        if send_error(socket, code, Some(sequence)).await.is_err() {
                            return;
                        }
                    }
                    Message::Close(_) => return,
                }
            }
        }
    }
}

fn command_sequence(message: &GameClientMessage) -> Option<Counter> {
    match message {
        GameClientMessage::Authenticate { .. } => None,
        GameClientMessage::CreateRoom { sequence }
        | GameClientMessage::JoinRoom { sequence, .. }
        | GameClientMessage::Ready { sequence, .. }
        | GameClientMessage::Movement { sequence, .. }
        | GameClientMessage::PlaceBomb { sequence, .. }
        | GameClientMessage::LeaveRoom { sequence, .. } => Some(*sequence),
    }
}

async fn send_error(
    socket: &mut WebSocket,
    code: ErrorCode,
    sequence: Option<Counter>,
) -> Result<(), ()> {
    send_game(socket, &GameServerMessage::Error { code, sequence }).await
}

async fn fail_and_close(socket: &mut WebSocket, code: ErrorCode, sequence: Option<Counter>) {
    let _ = send_error(socket, code, sequence).await;
    close(socket, 1008, "Policy violation").await;
}

async fn send_game(socket: &mut WebSocket, message: &GameServerMessage) -> Result<(), ()> {
    let text = encode_server_message(message).map_err(|_| ())?;
    send_raw(socket, Message::Text(text.into())).await
}

async fn send_raw(socket: &mut WebSocket, message: Message) -> Result<(), ()> {
    timeout(SEND_TIMEOUT, socket.send(message))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())
}

async fn close(socket: &mut WebSocket, code: u16, reason: &'static str) {
    let _ = send_raw(
        socket,
        Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        })),
    )
    .await;
}
