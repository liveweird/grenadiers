use std::time::Duration;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{CloseFrame, Message, WebSocket},
    },
    http::{HeaderMap, StatusCode, header::ORIGIN},
    response::Response,
};
use grenadiers_core::{ClientMessage, ServerMessage};
use tokio::{sync::OwnedSemaphorePermit, time::timeout};

use crate::{AppState, MAX_MESSAGE_BYTES, Problem};

const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(30);
const SEND_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) async fn websocket(
    State(state): State<AppState>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let origin = headers.get(ORIGIN).and_then(|value| value.to_str().ok());
    if origin.is_none_or(|value| !state.allowed_origins.contains(value)) {
        return Problem::new(StatusCode::FORBIDDEN, "Forbidden", "Origin is not allowed")
            .response();
    }
    if state.game_rooms.is_stopping() {
        return Problem::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Service Unavailable",
            "Server is shutting down",
        )
        .response();
    }
    let Ok(permit) = state.connections.clone().try_acquire_owned() else {
        return Problem::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Service Unavailable",
            "WebSocket connection limit reached",
        )
        .response();
    };
    let shutdown = state.game_rooms.shutdown();
    upgrade
        .max_frame_size(MAX_MESSAGE_BYTES)
        .max_message_size(MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| websocket_session(socket, permit, shutdown))
}

async fn websocket_session(
    mut socket: WebSocket,
    _permit: OwnedSemaphorePermit,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) {
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
