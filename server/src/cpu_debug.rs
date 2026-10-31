use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    Json,
    body::{Body, to_bytes},
    extract::{ConnectInfo, State, connect_info::MockConnectInfo},
    http::{Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use grenadiers_core::protocol::{RoomId, v14::ResumeToken};
use serde::Deserialize;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::{
    AppState, Problem,
    game::origin_allowed,
    peer_identity::{PeerKey, TransportPeer},
    rate_limit::TokenBucket,
    with_no_store,
};

mod contracts;
pub(crate) use contracts::*;

const MAX_BODY_BYTES: usize = 128;
const QUERY_TIMEOUT: Duration = Duration::from_millis(100);
const MAX_TRACKED_PEERS: usize = 4_096;
const IDLE_EVICTION_AGE: Duration = Duration::from_secs(60);
const MAX_CONCURRENT_QUERIES: usize = 4;

#[derive(Clone)]
pub(crate) struct CpuDebugGate {
    peers: Arc<Mutex<HashMap<PeerKey, PeerBudget>>>,
    concurrent: Arc<Semaphore>,
}

struct PeerBudget {
    bucket: TokenBucket,
    last_activity: Instant,
}

impl CpuDebugGate {
    pub(crate) fn new(_now: Instant) -> Self {
        Self {
            peers: Arc::new(Mutex::new(HashMap::new())),
            concurrent: Arc::new(Semaphore::new(MAX_CONCURRENT_QUERIES)),
        }
    }

    fn acquire(&self, peer: PeerKey, now: Instant) -> Result<OwnedSemaphorePermit, GateError> {
        let mut peers = self.peers.lock().map_err(|_| GateError::Unavailable)?;
        if !peers.contains_key(&peer) {
            if peers.len() >= MAX_TRACKED_PEERS {
                peers.retain(|_, budget| {
                    now.saturating_duration_since(budget.last_activity) < IDLE_EVICTION_AGE
                });
            }
            if peers.len() >= MAX_TRACKED_PEERS {
                return Err(GateError::Unavailable);
            }
            peers.insert(
                peer,
                PeerBudget {
                    bucket: TokenBucket::new(now, 2.0, 2.0),
                    last_activity: now,
                },
            );
        }
        let budget = peers.get_mut(&peer).expect("peer budget was inserted");
        budget.last_activity = budget.last_activity.max(now);
        if !budget.bucket.take(now) {
            return Err(GateError::RateLimited);
        }
        drop(peers);
        self.concurrent
            .clone()
            .try_acquire_owned()
            .map_err(|_| GateError::RateLimited)
    }
}

#[derive(Debug)]
enum GateError {
    RateLimited,
    Unavailable,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CpuDebugRequest {
    room_id: RoomId,
}

pub(crate) async fn query(State(state): State<AppState>, request: Request<Body>) -> Response {
    if !state.cpu_debug_enabled {
        return problem(
            StatusCode::NOT_FOUND,
            "Not Found",
            "API route does not exist",
        );
    }
    let (parts, body) = request.into_parts();
    if parts.uri.query().is_some_and(|query| !query.is_empty()) {
        return problem(
            StatusCode::BAD_REQUEST,
            "Bad Request",
            "Query parameters are not supported",
        );
    }
    if !origin_allowed(&parts.headers, &state) {
        return problem(StatusCode::FORBIDDEN, "Forbidden", "Origin is not allowed");
    }
    if parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("application/json")
    {
        return problem(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported Media Type",
            "Content-Type must be application/json",
        );
    }
    let transport = parts
        .extensions
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|value| value.0)
        .or_else(|| {
            parts
                .extensions
                .get::<MockConnectInfo<std::net::SocketAddr>>()
                .map(|value| value.0)
        });
    let peer = match crate::resolve_request_peer(
        TransportPeer(transport),
        &parts.headers,
        &state,
        crate::admission::DenialOperation::DiagnosticUpgrade,
    ) {
        Ok(peer) => peer,
        Err(error) => return crate::peer_identity_response(error),
    };
    let _permit = match state.cpu_debug_gate.acquire(peer.key(), Instant::now()) {
        Ok(permit) => permit,
        Err(GateError::RateLimited) => return rate_response(),
        Err(GateError::Unavailable) => {
            return problem(
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                "CPU diagnostics are unavailable",
            );
        }
    };
    let token = match bearer_token(&parts.headers) {
        Some(token) => token,
        None => return unauthorized(),
    };
    let bytes = match tokio::time::timeout(QUERY_TIMEOUT, to_bytes(body, MAX_BODY_BYTES)).await {
        Ok(Ok(bytes)) => bytes,
        Ok(Err(_)) => {
            return problem(
                StatusCode::PAYLOAD_TOO_LARGE,
                "Payload Too Large",
                "Request body exceeds the CPU diagnostic limit",
            );
        }
        Err(_) => {
            return problem(
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                "CPU diagnostic request body timed out",
            );
        }
    };
    let input: CpuDebugRequest = match serde_json::from_slice(&bytes) {
        Ok(input) => input,
        Err(_) => {
            return problem(
                StatusCode::BAD_REQUEST,
                "Bad Request",
                "Request body is invalid",
            );
        }
    };
    match tokio::time::timeout(
        QUERY_TIMEOUT,
        state.game_rooms.cpu_debug(&input.room_id, token),
    )
    .await
    {
        Ok(Ok(response)) => with_no_store(Json(response).into_response()),
        Ok(Err(CpuDebugQueryError::Unauthorized)) => unauthorized(),
        Ok(Err(CpuDebugQueryError::Unavailable)) => {
            problem(StatusCode::NOT_FOUND, "Not Found", "Room is unavailable")
        }
        Err(_) => problem(
            StatusCode::SERVICE_UNAVAILABLE,
            "Service Unavailable",
            "CPU diagnostics timed out",
        ),
    }
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Option<ResumeToken> {
    let mut values = headers.get_all(header::AUTHORIZATION).iter();
    let value = values.next()?.to_str().ok()?;
    if values.next().is_some() {
        return None;
    }
    ResumeToken::new(value.strip_prefix("Bearer ")?.to_owned()).ok()
}

fn problem(status: StatusCode, title: &'static str, detail: &'static str) -> Response {
    with_no_store(Problem::new(status, title, detail).response())
}
fn unauthorized() -> Response {
    let mut response = problem(
        StatusCode::UNAUTHORIZED,
        "Unauthorized",
        "Room capability is invalid",
    );
    response
        .headers_mut()
        .insert(header::WWW_AUTHENTICATE, "Bearer".parse().unwrap());
    response
}
fn rate_response() -> Response {
    let mut response = problem(
        StatusCode::TOO_MANY_REQUESTS,
        "Too Many Requests",
        "CPU diagnostic rate limit reached",
    );
    response
        .headers_mut()
        .insert(header::RETRY_AFTER, "1".parse().unwrap());
    response
}

#[cfg(test)]
#[path = "cpu_debug/gate_tests.rs"]
mod gate_tests;
