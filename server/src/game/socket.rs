use std::time::{Duration, Instant as StdInstant};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{CloseFrame, Message, WebSocket},
    },
    http::{HeaderMap, StatusCode, Uri, header::RETRY_AFTER},
    response::Response,
};
use grenadiers_core::protocol::v10::{
    GAMEPLAY_PROTOCOL_VERSION, GameClientMessage, GameServerMessage, GameplayVersion,
    V10ErrorCode as ErrorCode, encode_server_message, parse_client_message,
};
use grenadiers_core::protocol::{Counter, MAX_CLIENT_MESSAGE_BYTES, ProtocolVersionInput};
use tokio::{
    sync::OwnedSemaphorePermit,
    time::{Instant, timeout, timeout_at},
};

use super::{http::origin_allowed, limits::InboundLimit, sessions::SessionClaimError};
use crate::{AppState, Problem, with_no_store};

const AUTH_TIMEOUT: Duration = Duration::from_secs(5);
const SEND_TIMEOUT: Duration = Duration::from_secs(5);
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
    if state.game_rooms.is_stopping() {
        return with_no_store(
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                "Server is shutting down",
            )
            .response(),
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
    let rooms = state.game_rooms;
    let shutdown = rooms.shutdown();
    upgrade
        .max_frame_size(MAX_CLIENT_MESSAGE_BYTES)
        .max_message_size(MAX_CLIENT_MESSAGE_BYTES)
        .write_buffer_size(0)
        .max_write_buffer_size(MAX_WRITE_BUFFER)
        .on_upgrade(move |socket| game_session(socket, permit, sessions, rooms, shutdown))
}

async fn game_session(
    mut socket: WebSocket,
    _permit: OwnedSemaphorePermit,
    sessions: std::sync::Arc<super::sessions::GameSessions>,
    rooms: std::sync::Arc<super::rooms::GameRooms>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    let mut inbound = InboundLimit::new(Instant::now());
    let deadline = Instant::now() + AUTH_TIMEOUT;
    let lease = loop {
        if *shutdown.borrow() {
            close(&mut socket, 1001, "Server shutting down").await;
            return;
        }
        let incoming = match tokio::select! {
            incoming = timeout_at(deadline, socket.recv()) => incoming,
            changed = shutdown.changed() => {
                let _ = changed;
                close(&mut socket, 1001, "Server shutting down").await;
                return;
            }
        } {
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
                            supported_version: ProtocolVersionInput::new(GAMEPLAY_PROTOCOL_VERSION),
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
    super::connection::authenticated_loop(&mut socket, &mut inbound, lease.connection_id(), rooms)
        .await;
    drop(lease);
}

pub(super) fn command_sequence(message: &GameClientMessage) -> Option<Counter> {
    match message {
        GameClientMessage::Authenticate { .. } => None,
        GameClientMessage::CreateRoom { sequence }
        | GameClientMessage::JoinRoom { sequence, .. }
        | GameClientMessage::ResumeRoom { sequence, .. }
        | GameClientMessage::Ready { sequence, .. }
        | GameClientMessage::UpdateSettings { sequence, .. }
        | GameClientMessage::Movement { sequence, .. }
        | GameClientMessage::ManualAction { sequence, .. }
        | GameClientMessage::LeaveRoom { sequence, .. }
        | GameClientMessage::Rematch { sequence, .. } => Some(*sequence),
    }
}

pub(super) async fn send_error(
    socket: &mut WebSocket,
    code: ErrorCode,
    sequence: Option<Counter>,
) -> Result<(), ()> {
    send_game(socket, &GameServerMessage::Error { code, sequence }).await
}

pub(super) async fn fail_and_close(
    socket: &mut WebSocket,
    code: ErrorCode,
    sequence: Option<Counter>,
) {
    let _ = send_error(socket, code, sequence).await;
    close(socket, 1008, "Policy violation").await;
}

pub(super) async fn send_game(
    socket: &mut WebSocket,
    message: &GameServerMessage,
) -> Result<(), ()> {
    let text = encode_server_message(message).map_err(|_| ())?;
    send_raw(socket, Message::Text(text.into())).await
}

pub(super) async fn send_raw(socket: &mut WebSocket, message: Message) -> Result<(), ()> {
    timeout(SEND_TIMEOUT, socket.send(message))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())
}

pub(super) async fn close(socket: &mut WebSocket, code: u16, reason: &'static str) {
    let _ = send_raw(
        socket,
        Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        })),
    )
    .await;
}
