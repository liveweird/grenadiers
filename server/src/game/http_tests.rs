use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use axum::{
    Router,
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{Request, StatusCode, header},
    response::Response,
};
use http_body_util::BodyExt;
use jsonschema::validator_for;
use serde_json::{Value, json};
use tower::ServiceExt;

use super::*;
use crate::peer_identity::{PeerKey, TrustedProxyIps};

const ORIGIN: &str = "http://localhost:5177";

fn test_state() -> AppState {
    AppState::without_database([ORIGIN.to_owned()])
}

fn test_app(state: AppState) -> Router {
    app(state, None).layer(MockConnectInfo(std::net::SocketAddr::from((
        [192, 0, 2, 1],
        1234,
    ))))
}

async fn body_json(response: Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

fn contract() -> Value {
    serde_json::from_str(include_str!("../../../api/openapi.json")).unwrap()
}

fn assert_valid(schema: &Value, instance: &Value) {
    let validator = validator_for(schema).unwrap();
    let errors = validator
        .iter_errors(instance)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "schema errors: {errors:?}");
}

async fn issue(state: AppState, origin: Option<&str>) -> Response {
    issue_path(state, "/api/v1/game/session", origin).await
}

async fn issue_forwarded(state: AppState, client: &str) -> Response {
    let request = Request::post("/api/v1/game/session")
        .header(header::ORIGIN, ORIGIN)
        .header("x-forwarded-for", client)
        .body(Body::empty())
        .unwrap();
    test_app(state).oneshot(request).await.unwrap()
}

async fn issue_path(state: AppState, path: &str, origin: Option<&str>) -> Response {
    let mut request = Request::post(path);
    if let Some(origin) = origin {
        request = request.header(header::ORIGIN, origin);
    }
    test_app(state)
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn assert_problem(response: Response, status: StatusCode, detail: &str) -> Value {
    assert_eq!(response.status(), status);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    let body = body_json(response).await;
    assert_valid(&contract()["components"]["schemas"]["Problem"], &body);
    assert_eq!(body["detail"], detail);
    assert_eq!(body.as_object().unwrap().len(), 4);
    body
}

#[tokio::test]
async fn session_route_returns_exact_contract_response_without_caching() {
    let response = issue(test_state(), Some(ORIGIN)).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");

    let body = body_json(response).await;
    let schema = &contract()["components"]["schemas"]["GameSession"];
    assert_valid(schema, &body);
    assert_eq!(body.as_object().unwrap().len(), 1);
    assert_eq!(body["token"].as_str().unwrap().len(), 64);
}

#[tokio::test]
async fn session_identity_failures_are_typed_and_not_cached() {
    let request = Request::post("/api/v1/game/session")
        .header(header::ORIGIN, ORIGIN)
        .body(Body::empty())
        .unwrap();
    let missing = app(test_state(), None).oneshot(request).await.unwrap();
    assert_problem(
        missing,
        StatusCode::SERVICE_UNAVAILABLE,
        "Connection identity is unavailable",
    )
    .await;

    let mut trusted = test_state();
    trusted.trusted_proxies =
        Arc::new(TrustedProxyIps::parse(Some("192.0.2.1")).expect("trusted test transport"));
    let request = Request::post("/api/v1/game/session")
        .header(header::ORIGIN, ORIGIN)
        .body(Body::empty())
        .unwrap();
    let invalid = test_app(trusted).oneshot(request).await.unwrap();
    assert_problem(
        invalid,
        StatusCode::BAD_REQUEST,
        "Forwarded client identity is invalid",
    )
    .await;
}

#[tokio::test]
async fn untrusted_forwarding_cannot_reset_the_direct_peer_budget() {
    let state = test_state();
    for index in 0..8 {
        assert_eq!(
            issue_forwarded(state.clone(), &format!("198.51.100.{}", index + 1))
                .await
                .status(),
            StatusCode::CREATED
        );
    }
    let denied = issue_forwarded(state, "203.0.113.250").await;
    assert_eq!(denied.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(denied.headers()[header::RETRY_AFTER], "1");
}

#[tokio::test]
async fn outstanding_session_quota_is_a_peer_429() {
    let state = test_state();
    let peer = PeerKey::Ipv4(std::net::Ipv4Addr::new(192, 0, 2, 1));
    let base = Instant::now();
    for index in 0..16 {
        state
            .game_sessions
            .issue_for_peer(peer, base + Duration::from_millis(index * 125))
            .unwrap();
    }
    let response = issue(state, Some(ORIGIN)).await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(response.headers()[header::RETRY_AFTER], "1");
    assert_problem(
        response,
        StatusCode::TOO_MANY_REQUESTS,
        "Outstanding session limit reached",
    )
    .await;
}

#[tokio::test]
async fn session_route_requires_an_exact_allowed_origin() {
    let expected = json!({
        "type": "about:blank",
        "title": "Forbidden",
        "status": 403,
        "detail": "Origin is not allowed"
    });
    for origin in [
        None,
        Some("https://attacker.invalid"),
        Some("http://localhost:5177/"),
    ] {
        let response = issue(test_state(), origin).await;
        let body = assert_problem(response, StatusCode::FORBIDDEN, "Origin is not allowed").await;
        assert_eq!(body, expected);
    }

    let response = issue(
        AppState::without_database(["null".to_owned()]),
        Some("null"),
    )
    .await;
    let body = assert_problem(response, StatusCode::FORBIDDEN, "Origin is not allowed").await;
    assert_eq!(body, expected);
}

#[tokio::test]
async fn session_route_rejects_duplicate_origin_and_query_before_entropy() {
    let calls = Arc::new(AtomicUsize::new(0));
    let observed_calls = Arc::clone(&calls);
    let mut state = test_state();
    state.game_sessions = Arc::new(GameSessions::with_random_fill(
        Instant::now(),
        move |bytes| {
            observed_calls.fetch_add(1, Ordering::Relaxed);
            bytes.fill(3);
            Ok(())
        },
    ));

    let mut duplicate = Request::post("/api/v1/game/session");
    duplicate = duplicate.header(header::ORIGIN, ORIGIN);
    duplicate = duplicate.header(header::ORIGIN, ORIGIN);
    let response = test_app(state.clone())
        .oneshot(duplicate.body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_problem(response, StatusCode::FORBIDDEN, "Origin is not allowed").await;

    let malformed = Request::post("/api/v1/game/session")
        .header(
            header::ORIGIN,
            header::HeaderValue::from_bytes(&[0xff]).unwrap(),
        )
        .body(Body::empty())
        .unwrap();
    let response = test_app(state.clone()).oneshot(malformed).await.unwrap();
    assert_problem(response, StatusCode::FORBIDDEN, "Origin is not allowed").await;

    let response = issue_path(
        state.clone(),
        "/api/v1/game/session?token=private-value",
        Some(ORIGIN),
    )
    .await;
    assert_problem(
        response,
        StatusCode::BAD_REQUEST,
        "Query parameters are not supported",
    )
    .await;
    for _ in 0..32 {
        assert_eq!(
            issue(state.clone(), None).await.status(),
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(calls.load(Ordering::Relaxed), 0);

    assert_eq!(
        issue(state, Some(ORIGIN)).await.status(),
        StatusCode::CREATED
    );
    assert_eq!(calls.load(Ordering::Relaxed), 1);
}

#[tokio::test]
async fn session_route_reports_rate_limit_without_private_values() {
    let mut state = test_state();
    state.trusted_proxies =
        Arc::new(TrustedProxyIps::parse(Some("192.0.2.1")).expect("trusted test transport"));
    for index in 0..32 {
        assert_eq!(
            issue_forwarded(state.clone(), &format!("198.51.100.{}", index / 8 + 1))
                .await
                .status(),
            StatusCode::CREATED
        );
    }
    let response = issue_forwarded(state, "198.51.100.5").await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response.headers()[header::RETRY_AFTER], "1");
    assert_eq!(
        assert_problem(
            response,
            StatusCode::TOO_MANY_REQUESTS,
            "Session creation rate limit reached"
        )
        .await,
        json!({
            "type": "about:blank",
            "title": "Too Many Requests",
            "status": 429,
            "detail": "Session creation rate limit reached"
        })
    );
}

#[tokio::test]
async fn session_route_hides_random_source_failures() {
    let mut state = test_state();
    state.game_sessions = Arc::new(GameSessions::with_random_fill(Instant::now(), |_| Err(())));
    let response = issue(state, Some(ORIGIN)).await;
    assert_eq!(
        assert_problem(
            response,
            StatusCode::SERVICE_UNAVAILABLE,
            "Session creation unavailable"
        )
        .await,
        json!({
            "type": "about:blank",
            "title": "Service Unavailable",
            "status": 503,
            "detail": "Session creation unavailable"
        })
    );
}

#[tokio::test]
async fn session_route_reports_capacity_without_exposing_registry_state() {
    let base = Instant::now() - Duration::from_secs(20);
    let next = AtomicUsize::new(0);
    let sessions = Arc::new(GameSessions::with_random_fill(base, move |bytes| {
        let value = next.fetch_add(1, Ordering::Relaxed);
        bytes.fill(value as u8);
        Ok(())
    }));
    for index in 0..128 {
        sessions
            .issue_for_peer(
                PeerKey::Ipv4(std::net::Ipv4Addr::from(0xc633_6400 + index as u32 / 16)),
                base + Duration::from_millis(index * 125),
            )
            .unwrap();
    }
    let mut state = test_state();
    state.game_sessions = sessions;
    let response = issue(state, Some(ORIGIN)).await;
    assert_problem(
        response,
        StatusCode::SERVICE_UNAVAILABLE,
        "Session capacity reached",
    )
    .await;
}
