use std::time::{Duration, Instant as StdInstant};

use axum::{
    body::Body,
    extract::{
        State, WebSocketUpgrade,
        ws::{CloseFrame, Message, WebSocket},
    },
    http::{HeaderMap, Request, StatusCode, header::RETRY_AFTER},
    middleware::Next,
    response::Response,
};
use grenadiers_core::{ClientMessage, ServerMessage};
use tokio::{
    sync::OwnedSemaphorePermit,
    time::{Instant, timeout},
};

use crate::{
    AppState, MAX_MESSAGE_BYTES, Problem,
    game::{InboundLimit, origin_allowed},
    with_no_store,
};

const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(30);
const SEND_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_WRITE_BUFFER: usize = 131_072;

pub(crate) async fn no_store(request: Request<Body>, next: Next) -> Response {
    with_no_store(next.run(request).await)
}

pub(crate) async fn websocket(
    State(state): State<AppState>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
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
    if !state.diagnostic_upgrade_limit.allow(StdInstant::now()) {
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
    let Ok(permit) = state.diagnostic_connections.clone().try_acquire_owned() else {
        return with_no_store(
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                "Diagnostic WebSocket connection limit reached",
            )
            .response(),
        );
    };
    let shutdown = state.game_rooms.shutdown();
    with_no_store(
        upgrade
            .max_frame_size(MAX_MESSAGE_BYTES)
            .max_message_size(MAX_MESSAGE_BYTES)
            .write_buffer_size(0)
            .max_write_buffer_size(MAX_WRITE_BUFFER)
            .on_upgrade(move |socket| websocket_session(socket, permit, shutdown)),
    )
}

async fn websocket_session(
    mut socket: WebSocket,
    _permit: OwnedSemaphorePermit,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
    let mut inbound = InboundLimit::new(Instant::now());
    if *shutdown.borrow() {
        close(&mut socket, 1001, "Server shutting down").await;
        return;
    }
    if send_json(&mut socket, &ServerMessage::welcome())
        .await
        .is_err()
    {
        return;
    }
    loop {
        let incoming = tokio::select! {
            incoming = timeout(INACTIVITY_TIMEOUT, socket.recv()) => {
                let Ok(incoming) = incoming else {
                    close(&mut socket, 1008, "Connection inactive").await;
                    return;
                };
                incoming
            }
            changed = shutdown.changed() => {
                let _ = changed;
                close(&mut socket, 1001, "Server shutting down").await;
                return;
            }
        };
        if !inbound.take(Instant::now()) {
            close(&mut socket, 1008, "Rate limit exceeded").await;
            return;
        }
        match incoming {
            Some(Ok(Message::Text(text))) => match serde_json::from_str::<ClientMessage>(&text) {
                Ok(ClientMessage::Ping { sequence }) => {
                    if send_json(&mut socket, &ServerMessage::Pong { sequence })
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                Err(_) => {
                    let _ = send_json(&mut socket, &ServerMessage::invalid_message()).await;
                    close(&mut socket, 1008, "Invalid client message").await;
                    return;
                }
            },
            Some(Ok(Message::Ping(data))) => {
                if !matches!(
                    timeout(SEND_TIMEOUT, socket.send(Message::Pong(data))).await,
                    Ok(Ok(()))
                ) {
                    return;
                }
            }
            Some(Ok(Message::Pong(_))) => {}
            Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return,
            Some(Ok(Message::Binary(_))) => {
                let _ = send_json(&mut socket, &ServerMessage::invalid_message()).await;
                close(&mut socket, 1008, "Invalid client message").await;
                return;
            }
        }
    }
}

async fn send_json(socket: &mut WebSocket, message: &ServerMessage) -> Result<(), ()> {
    let text = serde_json::to_string(message).map_err(|_| ())?;
    timeout(SEND_TIMEOUT, socket.send(Message::Text(text.into())))
        .await
        .map_err(|_| ())?
        .map_err(|_| ())
}

async fn close(socket: &mut WebSocket, code: u16, reason: &'static str) {
    let _ = timeout(
        SEND_TIMEOUT,
        socket.send(Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        }))),
    )
    .await;
}
