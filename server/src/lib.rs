#![forbid(unsafe_code)]

use std::{
    collections::HashSet,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{
        HeaderMap, Request, StatusCode, Uri,
        header::{ALLOW, CACHE_CONTROL, CONTENT_TYPE},
    },
    middleware,
    response::{IntoResponse, Response},
    routing::{any, get, post},
};
use serde::Serialize;
use sqlx::PgPool;
use tokio::{sync::Semaphore, time::timeout};

pub(crate) mod admission;
mod browser_response;
mod configuration;
mod diagnostics;
mod game;
pub(crate) mod peer_identity;
mod rate_limit;
mod round_seed;
mod shutdown;
mod websocket_policy;

pub use configuration::{Config, StartupError, StartupWarning};
use game::{GameRooms, GameSessions, UpgradeLimit, game_websocket, issue_session};
pub use shutdown::{ShutdownOutcome, run_with_shutdown};
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
pub(crate) const MAX_CONNECTIONS: usize = 128;
pub(crate) const MAX_DIAGNOSTIC_CONNECTIONS: usize = 16;
const MAX_MESSAGE_BYTES: usize = 4096;
const READINESS_TIMEOUT: Duration = Duration::from_secs(2);
const SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct AppState {
    database: Option<PgPool>,
    allowed_origins: Arc<HashSet<String>>,
    connections: Arc<Semaphore>,
    diagnostic_connections: Arc<Semaphore>,
    diagnostic_upgrade_limit: Arc<UpgradeLimit>,
    game_sessions: Arc<GameSessions>,
    game_rooms: Arc<GameRooms>,
    trusted_proxies: Arc<peer_identity::TrustedProxyIps>,
    gameplay_admission: admission::GameplayAdmission,
    diagnostic_admission: admission::DiagnosticAdmission,
    denial_telemetry: Arc<admission::DenialTelemetry>,
}

impl AppState {
    pub fn begin_shutdown(&self) {
        self.game_rooms.stop();
    }

    pub async fn wait_for_shutdown(&self) -> bool {
        timeout(SHUTDOWN_DRAIN_TIMEOUT, self.drain_shutdown())
            .await
            .is_ok()
    }

    async fn drain_shutdown(&self) {
        let gameplay_permits = self
            .connections
            .clone()
            .acquire_many_owned(MAX_CONNECTIONS as u32);
        let diagnostic_permits = self
            .diagnostic_connections
            .clone()
            .acquire_many_owned(MAX_DIAGNOSTIC_CONNECTIONS as u32);
        let (gameplay_permits, diagnostic_permits, ()) = tokio::join!(
            gameplay_permits,
            diagnostic_permits,
            self.game_rooms.wait_empty()
        );
        drop(gameplay_permits.expect("gameplay connection semaphore remains open"));
        drop(diagnostic_permits.expect("diagnostic connection semaphore remains open"));
    }
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
        .route(
            "/api/v1/ws",
            get(diagnostics::websocket)
                .head(api_method_not_allowed)
                .layer(middleware::from_fn(no_store_middleware)),
        )
        .route("/api/v1/game/session", post(issue_session))
        .route(
            "/api/v1/game/ws",
            get(game_websocket)
                .head(api_method_not_allowed)
                .layer(middleware::from_fn(no_store_middleware)),
        )
        .route("/api", any(api_not_found))
        .route("/api/", any(api_not_found))
        .route("/api/{*path}", any(api_not_found))
        .method_not_allowed_fallback(api_method_not_allowed)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| http_request_span(request)),
        )
        .with_state(state);

    let router = match web_dist_dir {
        Some(directory) => router.fallback_service(
            ServeDir::new(directory.clone()).fallback(ServeFile::new(directory.join("index.html"))),
        ),
        None => router,
    };

    router.layer(middleware::from_fn(browser_response::headers_and_cache))
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

fn resolve_request_peer(
    transport: peer_identity::TransportPeer,
    headers: &HeaderMap,
    state: &AppState,
    operation: admission::DenialOperation,
) -> Result<peer_identity::PeerIdentity, peer_identity::PeerIdentityError> {
    peer_identity::resolve_peer_identity(transport.0, headers, &state.trusted_proxies).inspect_err(
        |_| {
            state.denial_telemetry.record(
                operation,
                admission::DenialReason::InvalidProxyIdentity,
                Instant::now(),
            );
        },
    )
}

fn peer_identity_response(error: peer_identity::PeerIdentityError) -> Response {
    let (status, title, detail) = match error {
        peer_identity::PeerIdentityError::Unavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Service Unavailable",
            "Connection identity is unavailable",
        ),
        peer_identity::PeerIdentityError::InvalidForwardedFor => (
            StatusCode::BAD_REQUEST,
            "Bad Request",
            "Forwarded client identity is invalid",
        ),
    };
    with_no_store(Problem::new(status, title, detail).response())
}

async fn no_store_middleware(request: Request<Body>, next: middleware::Next) -> Response {
    with_no_store(next.run(request).await)
}

async fn api_not_found() -> Response {
    with_no_store(
        Problem::new(
            StatusCode::NOT_FOUND,
            "Not Found",
            "API route does not exist",
        )
        .response(),
    )
}

async fn api_method_not_allowed(uri: Uri) -> Response {
    let mut response = with_no_store(
        Problem::new(
            StatusCode::METHOD_NOT_ALLOWED,
            "Method Not Allowed",
            "HTTP method is not supported for this API route",
        )
        .response(),
    );
    if matches!(uri.path(), "/api/v1/ws" | "/api/v1/game/ws") {
        response
            .headers_mut()
            .insert(ALLOW, "GET".parse().expect("valid allow header"));
    }
    response
}

#[cfg(test)]
mod admission_telemetry_tests;
#[cfg(test)]
mod browser_response_tests;
#[cfg(test)]
mod diagnostics_tests;
#[cfg(test)]
#[path = "game/http_tests.rs"]
mod game_http_tests;
#[cfg(test)]
#[path = "game/socket_tests.rs"]
mod game_socket_tests;
#[cfg(test)]
mod http_fallback_tests;
#[cfg(test)]
mod round_seed_tests;
#[cfg(test)]
mod session_listener_tests;
#[cfg(test)]
mod shutdown_coordinator_tests;
#[cfg(test)]
mod shutdown_listener_test_support;
#[cfg(test)]
mod shutdown_listener_tests;
#[cfg(test)]
mod shutdown_tests;
#[cfg(test)]
mod startup_tests;
#[cfg(test)]
mod tests;
