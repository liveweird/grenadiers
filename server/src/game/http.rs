use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, Uri, header::ORIGIN},
    response::{IntoResponse, Response},
};

use super::sessions::SessionIssueError;
use crate::{AppState, Problem, with_no_store};

pub(crate) async fn issue_session(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
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

    match state.game_sessions.issue(std::time::Instant::now()) {
        Ok(session) => with_no_store((StatusCode::CREATED, Json(session)).into_response()),
        Err(SessionIssueError::RateLimited) => {
            let mut response = Problem::new(
                StatusCode::TOO_MANY_REQUESTS,
                "Too Many Requests",
                "Session creation rate limit reached",
            )
            .response();
            response.headers_mut().insert(
                axum::http::header::RETRY_AFTER,
                "1".parse().expect("valid retry after"),
            );
            with_no_store(response)
        }
        Err(SessionIssueError::CapacityReached) => with_no_store(
            Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "Service Unavailable",
                "Session capacity reached",
            )
            .response(),
        ),
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

pub(super) fn origin_allowed(headers: &HeaderMap, state: &AppState) -> bool {
    let mut origins = headers.get_all(ORIGIN).iter();
    let origin = origins.next().and_then(|value| value.to_str().ok());
    origins.next().is_none()
        && origin.is_some_and(|value| value != "null" && state.allowed_origins.contains(value))
}
