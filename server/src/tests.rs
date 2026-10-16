use std::{future::Future, time::Duration};

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use futures_util::{SinkExt, StreamExt};
use http_body_util::BodyExt;
use jsonschema::validator_for;
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use tokio::{net::TcpListener, task::JoinHandle, time::timeout as time_limit};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        Error as WebSocketError, Message as TungsteniteMessage, client::IntoClientRequest,
        protocol::frame::coding::CloseCode,
    },
};
use tower::ServiceExt;

use super::*;

const ORIGIN: &str = "http://localhost:5177";
const TEST_TIMEOUT: Duration = Duration::from_secs(2);

fn state() -> AppState {
    AppState::without_database([ORIGIN.to_owned()])
}

async fn body_json(response: Response) -> Value {
    serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap()
}

fn openapi_component(name: &str) -> Value {
    let contract: Value = serde_json::from_str(include_str!("../../api/openapi.json")).unwrap();
    contract["components"]["schemas"][name].clone()
}

fn realtime_contract() -> Value {
    serde_json::from_str(include_str!("../../api/realtime.schema.json")).unwrap()
}

fn assert_valid(schema: &Value, instance: &Value) {
    let validator = validator_for(schema).unwrap();
    let errors = validator
        .iter_errors(instance)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "schema errors: {errors:?}");
}

async fn within<T>(future: impl Future<Output = T>) -> T {
    time_limit(TEST_TIMEOUT, future)
        .await
        .expect("operation exceeded test timeout")
}

#[test]
fn blank_database_url_disables_postgres() {
    assert!(database_pool(None).unwrap().is_none());
    assert!(database_pool(Some(String::new())).unwrap().is_none());
    assert!(database_pool(Some(" \t ".to_owned())).unwrap().is_none());
}

#[test]
fn openapi_declares_every_implemented_http_status() {
    let contract: Value = serde_json::from_str(include_str!("../../api/openapi.json")).unwrap();
    let cases = [
        ("/api/v1/health", ["200"].as_slice()),
        ("/api/v1/ready", ["200", "503"].as_slice()),
        ("/api/v1/ws", ["101", "400", "403", "503"].as_slice()),
    ];
    for (path, statuses) in cases {
        for status in statuses {
            assert!(
                contract["paths"][path]["get"]["responses"]
                    .get(*status)
                    .is_some(),
                "{path} is missing status {status}"
            );
        }
    }
}

#[tokio::test]
async fn health_and_ready_without_database_match_contract() {
    for path in ["/api/v1/health", "/api/v1/ready"] {
        let response = app(state(), None)
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_valid(&openapi_component("Status"), &body_json(response).await);
    }
}

#[tokio::test]
async fn unavailable_database_returns_problem() {
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(50))
        .connect_lazy("postgres://postgres:postgres@127.0.0.1:9/grenadiers")
        .unwrap();
    let mut unavailable = state();
    unavailable.database = Some(pool);
    let response = within(
        app(unavailable, None).oneshot(Request::get("/api/v1/ready").body(Body::empty()).unwrap()),
    )
    .await
    .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/problem+json"
    );
    assert_valid(&openapi_component("Problem"), &body_json(response).await);
}

#[tokio::test]
async fn websocket_rejections_cover_bad_upgrade_origin_and_capacity() {
    let malformed = app(state(), None)
        .oneshot(
            Request::get("/api/v1/ws")
                .header(header::ORIGIN, ORIGIN)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);

    let (url, server) = start_server(state()).await;
    for origin in [None, Some("https://attacker.invalid")] {
        let forbidden = rejected_handshake(&url, origin).await;
        assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
        assert_valid(
            &openapi_component("Problem"),
            &serde_json::from_slice(forbidden.body().as_deref().unwrap()).unwrap(),
        );
    }
    server.abort();

    let saturated = state();
    let _permits = saturated
        .connections
        .clone()
        .acquire_many_owned(MAX_CONNECTIONS.try_into().unwrap())
        .await
        .unwrap();
    let (url, server) = start_server(saturated).await;
    let response = rejected_handshake(&url, Some(ORIGIN)).await;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_valid(
        &openapi_component("Problem"),
        &serde_json::from_slice(response.body().as_deref().unwrap()).unwrap(),
    );
    server.abort();
}

#[tokio::test]
async fn unknown_api_route_is_problem_instead_of_spa_fallback() {
    for request in [
        Request::get("/api/v1/missing").body(Body::empty()).unwrap(),
        Request::post("/api/v1/missing")
            .body(Body::empty())
            .unwrap(),
        Request::delete("/api").body(Body::empty()).unwrap(),
        Request::post("/api/").body(Body::empty()).unwrap(),
    ] {
        let response = app(state(), Some("web/dist".into()))
            .oneshot(request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_valid(&openapi_component("Problem"), &body_json(response).await);
    }
}

type ClientSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn start_server(state: AppState) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, app(state, None)).await.unwrap();
    });
    (format!("ws://{address}/api/v1/ws"), task)
}

async fn rejected_handshake(
    url: &str,
    origin: Option<&str>,
) -> tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>> {
    let mut request = url.into_client_request().unwrap();
    if let Some(origin) = origin {
        request
            .headers_mut()
            .insert(header::ORIGIN, origin.parse().unwrap());
    }
    match within(connect_async(request)).await {
        Err(WebSocketError::Http(response)) => response,
        Err(error) => panic!("expected HTTP rejection, got {error}"),
        Ok(_) => panic!("expected HTTP rejection, got an upgraded WebSocket"),
    }
}

async fn connect(url: &str) -> ClientSocket {
    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert(header::ORIGIN, ORIGIN.parse().unwrap());
    let (socket, response) = within(connect_async(request)).await.unwrap();
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    socket
}

async fn next_json(socket: &mut ClientSocket) -> Value {
    let message = within(socket.next()).await.unwrap().unwrap();
    serde_json::from_str(message.to_text().unwrap()).unwrap()
}

#[tokio::test]
async fn live_sockets_are_concurrent_and_enforce_the_protocol() {
    let (url, server) = start_server(state()).await;
    let (mut first, mut second) = tokio::join!(connect(&url), connect(&url));
    let contract = realtime_contract();

    for socket in [&mut first, &mut second] {
        assert_valid(&contract, &next_json(socket).await);
    }

    within(first.send(TungsteniteMessage::Text(
        json!({"type": "ping", "sequence": 42}).to_string().into(),
    )))
    .await
    .unwrap();
    assert_valid(
        &realtime_contract()["$defs"]["Ping"],
        &json!({"type": "ping", "sequence": 42}),
    );
    let pong = next_json(&mut first).await;
    assert_eq!(pong, json!({"type": "pong", "sequence": 42}));
    assert_valid(&contract, &pong);

    within(second.send(TungsteniteMessage::Text("not json".into())))
        .await
        .unwrap();
    let error = next_json(&mut second).await;
    assert_eq!(error["type"], "error");
    assert_valid(&contract, &error);
    let close = within(second.next()).await.unwrap().unwrap();
    match close {
        TungsteniteMessage::Close(Some(frame)) => assert_eq!(frame.code, CloseCode::Policy),
        other => panic!("expected policy close, got {other:?}"),
    }

    within(first.close(None)).await.unwrap();
    server.abort();
}

#[tokio::test]
async fn live_socket_rejects_binary_and_oversized_messages() {
    let (url, server) = start_server(state()).await;

    let mut binary = connect(&url).await;
    assert_eq!(next_json(&mut binary).await["type"], "welcome");
    within(binary.send(TungsteniteMessage::Binary(vec![1, 2, 3].into())))
        .await
        .unwrap();
    let error = next_json(&mut binary).await;
    assert_eq!(error["type"], "error");
    assert_valid(&realtime_contract(), &error);
    match within(binary.next()).await {
        Some(Ok(TungsteniteMessage::Close(Some(frame)))) => {
            assert_eq!(frame.code, CloseCode::Policy);
        }
        other => panic!("expected binary-message policy close, got {other:?}"),
    }

    let mut oversized = connect(&url).await;
    assert_eq!(next_json(&mut oversized).await["type"], "welcome");
    within(oversized.send(TungsteniteMessage::Text(
        "x".repeat(MAX_MESSAGE_BYTES + 1).into(),
    )))
    .await
    .unwrap();
    match within(oversized.next()).await {
        Some(Ok(TungsteniteMessage::Close(Some(frame)))) => {
            assert_eq!(frame.code, CloseCode::Size);
        }
        Some(Err(_)) | None => {}
        other => panic!("oversized message was not rejected: {other:?}"),
    }

    server.abort();
}
