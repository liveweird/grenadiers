use std::time::{Duration, Instant as StdInstant};

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{CloseFrame, Message, WebSocket},
    },
    http::{HeaderMap, StatusCode, Uri, header::RETRY_AFTER},
    response::Response,
};
use grenadiers_core::protocol::v13::{
    GAMEPLAY_PROTOCOL_VERSION, GameServerMessage, GameplayVersion, V11ErrorCode as ErrorCode,
    encode_server_message, parse_client_message,
};
use grenadiers_core::protocol::{Counter, MAX_CLIENT_MESSAGE_BYTES, ProtocolVersionInput};
use tokio::{
    sync::OwnedSemaphorePermit,
    time::{Instant, timeout, timeout_at},
};

use super::{
    command::InboundMessage, http::origin_allowed, limits::InboundLimit,
    sessions::SessionClaimError,
};
use crate::{AppState, Problem, with_no_store};
use crate::{
    admission::{
        DenialOperation, DenialReason, DenialTelemetry, GameplayAdmission, GameplayOperation,
        GameplaySocketPermit, PolicyDenial,
    },
    peer_identity::{PeerKey, TransportPeer},
    peer_identity_response, resolve_request_peer,
};

const AUTH_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_WRITE_BUFFER: usize = 131_072;

pub(crate) async fn game_websocket(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
    transport: TransportPeer,
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
        state.denial_telemetry.record(
            DenialOperation::GameplayUpgrade,
            DenialReason::OriginRejected,
            StdInstant::now(),
        );
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
    let peer = match resolve_request_peer(
        transport,
        &headers,
        &state,
        DenialOperation::GameplayUpgrade,
    ) {
        Ok(peer) => peer,
        Err(error) => return peer_identity_response(error),
    };
    let now = StdInstant::now();
    if let Err(denial) = state
        .gameplay_admission
        .check(peer.key(), GameplayOperation::Upgrade, now)
    {
        return policy_response(&state, denial);
    }
    let peer_permit = match state.gameplay_admission.acquire_socket(peer.key(), now) {
        Ok(permit) => permit,
        Err(denial) => return policy_response(&state, denial),
    };
    if !state.game_sessions.allow_upgrade(now) {
        state.denial_telemetry.record(
            DenialOperation::GameplayUpgrade,
            DenialReason::GlobalRate,
            now,
        );
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
        state.denial_telemetry.record(
            DenialOperation::GameplayUpgrade,
            DenialReason::GlobalCapacity,
            now,
        );
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
    let admission = state.gameplay_admission;
    let telemetry = state.denial_telemetry;
    let shutdown = rooms.shutdown();
    upgrade
        .max_frame_size(MAX_CLIENT_MESSAGE_BYTES)
        .max_message_size(MAX_CLIENT_MESSAGE_BYTES)
        .write_buffer_size(0)
        .max_write_buffer_size(MAX_WRITE_BUFFER)
        .on_upgrade(move |socket| {
            game_session(
                socket,
                GameSessionContext {
                    _global_permit: permit,
                    _peer_permit: peer_permit,
                    peer_key: peer.key(),
                    sessions,
                    rooms,
                    admission,
                    telemetry,
                    shutdown,
                },
            )
        })
}

struct GameSessionContext {
    _global_permit: OwnedSemaphorePermit,
    _peer_permit: GameplaySocketPermit,
    peer_key: PeerKey,
    sessions: std::sync::Arc<super::sessions::GameSessions>,
    rooms: std::sync::Arc<super::rooms::GameRooms>,
    admission: GameplayAdmission,
    telemetry: std::sync::Arc<DenialTelemetry>,
    shutdown: tokio::sync::watch::Receiver<bool>,
}

async fn game_session(mut socket: WebSocket, context: GameSessionContext) {
    let GameSessionContext {
        _global_permit,
        _peer_permit,
        peer_key,
        sessions,
        rooms,
        admission,
        telemetry,
        mut shutdown,
    } = context;
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
                let (protocol_version, token) = match InboundMessage::from(message) {
                    InboundMessage::Authenticate {
                        protocol_version,
                        token,
                    } => (protocol_version, token),
                    InboundMessage::Command(command) => {
                        fail_and_close(
                            &mut socket,
                            ErrorCode::NotAuthenticated,
                            Some(command.sequence()),
                        )
                        .await;
                        return;
                    }
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
                        telemetry.record(
                            DenialOperation::GameplayUpgrade,
                            match error {
                                SessionClaimError::CapacityReached => DenialReason::GlobalCapacity,
                                SessionClaimError::AuthenticationFailed
                                | SessionClaimError::ConnectionInUse => {
                                    DenialReason::AuthenticationFailed
                                }
                            },
                            StdInstant::now(),
                        );
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
    super::connection::authenticated_loop(
        &mut socket,
        &mut inbound,
        lease.connection_id(),
        peer_key,
        rooms,
        admission,
        telemetry,
    )
    .await;
    drop(lease);
}

fn policy_response(state: &AppState, denial: PolicyDenial) -> Response {
    state
        .denial_telemetry
        .record(denial.operation, denial.reason, StdInstant::now());
    if denial.reason == DenialReason::PeerRate {
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
    with_no_store(
        Problem::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Service Unavailable",
            "Peer connection capacity reached",
        )
        .response(),
    )
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
    timeout(crate::websocket_policy::SEND_TIMEOUT, socket.send(message))
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
