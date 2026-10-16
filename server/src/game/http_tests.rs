use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
    response::Response,
};
use http_body_util::BodyExt;
use jsonschema::validator_for;
use serde_json::{Value, json};
use tower::ServiceExt;
use tracing::{Subscriber, field::Visit, span::Attributes};
use tracing_subscriber::{Layer, layer::SubscriberExt};

use super::*;

const ORIGIN: &str = "http://localhost:5177";

fn test_state() -> AppState {
    AppState::without_database([ORIGIN.to_owned()])
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

async fn issue_path(state: AppState, path: &str, origin: Option<&str>) -> Response {
    let mut request = Request::post(path);
    if let Some(origin) = origin {
        request = request.header(header::ORIGIN, origin);
    }
    app(state, None)
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
    let response = app(state.clone(), None)
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
    let response = app(state.clone(), None).oneshot(malformed).await.unwrap();
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
    let state = test_state();
    for _ in 0..32 {
        assert_eq!(
            issue(state.clone(), Some(ORIGIN)).await.status(),
            StatusCode::CREATED
        );
    }
    let response = issue(state, Some(ORIGIN)).await;
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
            .issue(base + Duration::from_millis(index * 125))
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

#[derive(Clone)]
struct CapturedFields(Arc<Mutex<Vec<String>>>);

impl<Registry> Layer<Registry> for CapturedFields
where
    Registry: Subscriber,
{
    fn on_new_span(
        &self,
        attributes: &Attributes<'_>,
        _id: &tracing::span::Id,
        _context: tracing_subscriber::layer::Context<'_, Registry>,
    ) {
        attributes.record(&mut FieldVisitor(Arc::clone(&self.0)));
    }
}

struct FieldVisitor(Arc<Mutex<Vec<String>>>);

impl Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0
            .lock()
            .unwrap()
            .push(format!("{}={value:?}", field.name()));
    }
}

#[test]
fn trace_spans_exclude_query_credentials() {
    const CREDENTIAL: &str = "private-query-credential";
    let fields = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::registry().with(CapturedFields(Arc::clone(&fields)));
    let request = Request::post(format!("/api/v1/game/session?token={CREDENTIAL}"))
        .body(Body::empty())
        .unwrap();
    tracing::subscriber::with_default(subscriber, || drop(http_request_span(&request)));

    let captured = fields.lock().unwrap().join(" ");
    assert!(captured.contains("/api/v1/game/session"));
    assert!(!captured.contains(CREDENTIAL));
    assert!(!captured.contains("token="));
}
