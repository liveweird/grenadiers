use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, Uri, header::ORIGIN},
    response::{IntoResponse, Response},
};

use super::sessions::SessionIssueError;
use crate::{
    AppState, Problem,
    admission::{DenialOperation, DenialReason, GameplayOperation, PolicyDenial},
    peer_identity::TransportPeer,
    peer_identity_response, resolve_request_peer, with_no_store,
};

pub(crate) async fn issue_session(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
    transport: TransportPeer,
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
            DenialOperation::Mint,
            DenialReason::OriginRejected,
            std::time::Instant::now(),
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

    let peer = match resolve_request_peer(transport, &headers, &state, DenialOperation::Mint) {
        Ok(peer) => peer,
        Err(error) => return peer_identity_response(error),
    };
    let now = std::time::Instant::now();
    if let Err(denial) = state
        .gameplay_admission
        .check(peer.key(), GameplayOperation::Mint, now)
    {
        return policy_response(&state, denial);
    }

    match state.game_sessions.issue_for_peer(peer.key(), now) {
        Ok(session) => with_no_store((StatusCode::CREATED, Json(session)).into_response()),
        Err(SessionIssueError::PeerCapacity) => {
            state
                .denial_telemetry
                .record(DenialOperation::Mint, DenialReason::PeerCapacity, now);
            rate_response("Outstanding session limit reached")
        }
        Err(SessionIssueError::RateLimited) => {
            state
                .denial_telemetry
                .record(DenialOperation::Mint, DenialReason::GlobalRate, now);
            rate_response("Session creation rate limit reached")
        }
        Err(SessionIssueError::CapacityReached) => {
            state
                .denial_telemetry
                .record(DenialOperation::Mint, DenialReason::GlobalCapacity, now);
            with_no_store(
                Problem::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Service Unavailable",
                    "Session capacity reached",
                )
                .response(),
            )
        }
        Err(SessionIssueError::CreationUnavailable) => with_no_store(
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                "Session creation unavailable",
            )
            .response(),
        ),
    }
}

fn policy_response(state: &AppState, denial: PolicyDenial) -> Response {
    state
        .denial_telemetry
        .record(denial.operation, denial.reason, std::time::Instant::now());
    match denial.reason {
        DenialReason::PeerRate => rate_response("Session creation rate limit reached"),
        DenialReason::TrackingCapacity | DenialReason::PeerCapacity => with_no_store(
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                "Peer admission capacity reached",
            )
            .response(),
        ),
        _ => unreachable!("peer policy returns only peer or tracking denials"),
    }
}

fn rate_response(detail: &'static str) -> Response {
    let mut response =
        Problem::new(StatusCode::TOO_MANY_REQUESTS, "Too Many Requests", detail).response();
    response.headers_mut().insert(
        axum::http::header::RETRY_AFTER,
        "1".parse().expect("valid retry after"),
    );
    with_no_store(response)
}

pub(crate) fn origin_allowed(headers: &HeaderMap, state: &AppState) -> bool {
    let mut origins = headers.get_all(ORIGIN).iter();
    let origin = origins.next().and_then(|value| value.to_str().ok());
    origins.next().is_none()
        && origin.is_some_and(|value| value != "null" && state.allowed_origins.contains(value))
}
