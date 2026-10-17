#![forbid(unsafe_code)]

use std::{collections::HashSet, path::PathBuf, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{CloseFrame, Message, WebSocket},
    },
    http::{
        HeaderMap, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE, ORIGIN},
    },
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use grenadiers_core::{ClientMessage, ServerMessage};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::{
    sync::{OwnedSemaphorePermit, Semaphore},
    time::timeout,
};

mod game;
mod round_seed;

use game::{GameRooms, GameSessions, game_websocket, issue_session};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

const DEFAULT_ORIGINS: [&str; 4] = [
    "http://localhost:5177",
    "http://127.0.0.1:5177",
    "http://localhost:8085",
    "http://127.0.0.1:8085",
];
const MAX_CONNECTIONS: usize = 128;
const MAX_MESSAGE_BYTES: usize = 4096;
const READINESS_TIMEOUT: Duration = Duration::from_secs(2);
const INACTIVITY_TIMEOUT: Duration = Duration::from_secs(30);
const SEND_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct AppState {
    database: Option<PgPool>,
    allowed_origins: Arc<HashSet<String>>,
    connections: Arc<Semaphore>,
    game_sessions: Arc<GameSessions>,
    game_rooms: Arc<GameRooms>,
}

impl AppState {
    #[must_use]
    pub fn without_database(allowed_origins: impl IntoIterator<Item = String>) -> Self {
        Self {
            database: None,
            allowed_origins: Arc::new(allowed_origins.into_iter().collect()),
            connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
            game_sessions: Arc::new(GameSessions::new(std::time::Instant::now())),
            game_rooms: GameRooms::new(std::time::Instant::now()),
        }
    }

    pub fn from_environment() -> Result<Self, sqlx::Error> {
        Self::from_configuration(
            std::env::var("ALLOWED_ORIGINS").ok(),
            std::env::var("DATABASE_URL").ok(),
        )
    }

    fn from_configuration(
        allowed_origins: Option<String>,
        database_url: Option<String>,
    ) -> Result<Self, sqlx::Error> {
        let allowed_origins = allowed_origins.map_or_else(
            || DEFAULT_ORIGINS.iter().map(ToString::to_string).collect(),
            |value| {
                value
                    .split(',')
                    .map(str::trim)
                    .filter(|origin| !origin.is_empty())
                    .map(ToString::to_string)
                    .collect()
            },
        );
        let database = database_pool(database_url)?;
        Ok(Self {
            database,
            allowed_origins: Arc::new(allowed_origins),
            connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
            game_sessions: Arc::new(GameSessions::new(std::time::Instant::now())),
            game_rooms: GameRooms::new(std::time::Instant::now()),
        })
    }
}

fn database_pool(database_url: Option<String>) -> Result<Option<PgPool>, sqlx::Error> {
    let Some(database_url) = database_url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
    else {
        return Ok(None);
    };
    PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(READINESS_TIMEOUT)
        .connect_lazy(database_url)
        .map(Some)
}

#[derive(Debug, Serialize)]
pub struct StatusBody {
    status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Problem {
    #[serde(rename = "type")]
    problem_type: &'static str,
    title: &'static str,
    status: u16,
    detail: String,
}

impl Problem {
    fn new(status: StatusCode, title: &'static str, detail: impl Into<String>) -> Self {
        Self {
            problem_type: "about:blank",
            title,
            status: status.as_u16(),
            detail: detail.into(),
        }
    }

    fn response(self) -> Response {
        let status = StatusCode::from_u16(self.status).expect("problem status is valid");
        let mut response = (status, Json(self)).into_response();
        response.headers_mut().insert(
            CONTENT_TYPE,
            "application/problem+json"
                .parse()
                .expect("valid content type"),
        );
        response
    }
}

pub fn app(state: AppState, web_dist_dir: Option<PathBuf>) -> Router {
    let router = Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/ready", get(ready))
        .route("/api/v1/ws", get(websocket))
        .route("/api/v1/game/session", post(issue_session))
        .route("/api/v1/game/ws", get(game_websocket))
        .route("/api", any(api_not_found))
        .route("/api/", any(api_not_found))
        .route("/api/{*path}", any(api_not_found))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| http_request_span(request)),
        )
        .with_state(state);

    match web_dist_dir {
        Some(directory) => router.fallback_service(
            ServeDir::new(directory.clone()).fallback(ServeFile::new(directory.join("index.html"))),
        ),
        None => router,
    }
}

fn http_request_span<B>(request: &axum::http::Request<B>) -> tracing::Span {
    tracing::info_span!(
        "http.request",
        method = %request.method(),
        path = request.uri().path()
    )
}

async fn health() -> Response {
    status_response()
}

async fn ready(State(state): State<AppState>) -> Response {
    let Some(database) = state.database else {
        return status_response();
    };
    let response = match timeout(
        READINESS_TIMEOUT,
        sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&database),
    )
    .await
    {
        Ok(Ok(1)) => status_response(),
        Ok(Ok(_)) | Ok(Err(_)) | Err(_) => Problem::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "Service Unavailable",
            "Database readiness check failed",
        )
        .response(),
    };
    with_no_store(response)
}

fn status_response() -> Response {
    with_no_store(Json(StatusBody { status: "ok" }).into_response())
}

fn with_no_store(mut response: Response) -> Response {
    response.headers_mut().insert(
        CACHE_CONTROL,
        "no-store".parse().expect("valid cache control"),
    );
    response
}

async fn websocket(
    State(state): State<AppState>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    let origin = headers.get(ORIGIN).and_then(|value| value.to_str().ok());
    if origin.is_none_or(|value| !state.allowed_origins.contains(value)) {
        return Problem::new(StatusCode::FORBIDDEN, "Forbidden", "Origin is not allowed")
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
    upgrade
        .max_frame_size(MAX_MESSAGE_BYTES)
        .max_message_size(MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| websocket_session(socket, permit))
}

async fn websocket_session(mut socket: WebSocket, _permit: OwnedSemaphorePermit) {
    if send_json(&mut socket, &ServerMessage::welcome())
        .await
        .is_err()
    {
        return;
    }
    loop {
        let Ok(incoming) = timeout(INACTIVITY_TIMEOUT, socket.recv()).await else {
            close_policy(&mut socket, "Connection inactive").await;
            return;
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
                    close_policy(&mut socket, "Invalid client message").await;
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
                close_policy(&mut socket, "Invalid client message").await;
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

async fn close_policy(socket: &mut WebSocket, reason: &'static str) {
    let _ = timeout(
        SEND_TIMEOUT,
        socket.send(Message::Close(Some(CloseFrame {
            code: 1008,
            reason: reason.into(),
        }))),
    )
    .await;
}

async fn api_not_found() -> Response {
    Problem::new(
        StatusCode::NOT_FOUND,
        "Not Found",
        "API route does not exist",
    )
    .response()
}

#[cfg(test)]
#[path = "game/http_tests.rs"]
mod game_http_tests;
#[cfg(test)]
#[path = "game/socket_tests.rs"]
mod game_socket_tests;
#[cfg(test)]
mod round_seed_tests;
#[cfg(test)]
mod tests;
