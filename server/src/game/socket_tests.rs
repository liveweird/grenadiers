use std::{future::Future, time::Duration};

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{net::TcpListener, task::JoinHandle, time::timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Error as WebSocketError, Message, client::IntoClientRequest},
};

use super::*;

const ORIGIN: &str = "http://localhost:5177";
const TEST_TIMEOUT: Duration = Duration::from_secs(2);

type ClientSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn within<T>(future: impl Future<Output = T>) -> T {
    timeout(TEST_TIMEOUT, future).await.expect("test timed out")
}

fn state() -> AppState {
    AppState::without_database([ORIGIN.to_owned()])
}

fn issue_token(state: &AppState) -> String {
    serde_json::to_value(
        state
            .game_sessions
            .issue(std::time::Instant::now())
            .unwrap(),
    )
    .unwrap()["token"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn start_server(state: AppState) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, app(state, None)).await.unwrap();
    });
    (format!("ws://{address}/api/v1/game/ws"), task)
}

fn diagnostic_url(game_url: &str) -> String {
    game_url.replace("/api/v1/game/ws", "/api/v1/ws")
}

async fn wait_for_available_connection(state: &AppState) {
    within(async {
        while state.connections.available_permits() == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await;
}

async fn connect(url: &str) -> ClientSocket {
    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert(axum::http::header::ORIGIN, ORIGIN.parse().unwrap());
    let (socket, response) = within(connect_async(request)).await.unwrap();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::SWITCHING_PROTOCOLS
    );
    socket
}

async fn rejected(url: &str, origins: &[&str]) -> axum::http::StatusCode {
    let mut request = url.into_client_request().unwrap();
    for origin in origins {
        request
            .headers_mut()
            .append(axum::http::header::ORIGIN, origin.parse().unwrap());
    }
    match within(connect_async(request)).await {
        Err(WebSocketError::Http(response)) => response.status(),
        other => panic!("expected rejected handshake, got {other:?}"),
    }
}

async fn send_json(socket: &mut ClientSocket, value: Value) {
    within(socket.send(Message::Text(value.to_string().into())))
        .await
        .unwrap();
}

async fn next_json(socket: &mut ClientSocket) -> Value {
    let message = within(socket.next()).await.unwrap().unwrap();
    serde_json::from_str(message.to_text().unwrap()).unwrap()
}

async fn authenticate(socket: &mut ClientSocket, token: &str, version: u16) -> Value {
    send_json(
        socket,
        json!({"type":"authenticate", "protocolVersion":version, "token":token}),
    )
    .await;
    next_json(socket).await
}

#[tokio::test]
async fn authenticates_and_consumes_fresh_sequences_before_semantics() {
    let state = state();
    let token = issue_token(&state);
    let (url, server) = start_server(state).await;
    let mut socket = connect(&url).await;
    assert_eq!(
        authenticate(&mut socket, &token, 8).await,
        json!({"type":"authenticated", "protocolVersion":8})
    );

    send_json(&mut socket, json!({"type":"createRoom", "sequence":1})).await;
    assert_eq!(next_json(&mut socket).await["type"], "admitted");
    send_json(&mut socket, json!({"type":"createRoom", "sequence":1})).await;
    assert_eq!(
        next_json(&mut socket).await,
        json!({"type":"error", "code":"staleSequence", "sequence":1})
    );
    send_json(
        &mut socket,
        json!({"type":"authenticate", "protocolVersion":8, "token":token}),
    )
    .await;
    assert_eq!(next_json(&mut socket).await["code"], "alreadyAuthenticated");
    server.abort();
}

#[tokio::test]
async fn version_precedes_claim_and_duplicate_cannot_revoke_owner() {
    let state = state();
    let token = issue_token(&state);
    let (url, server) = start_server(state).await;

    for version in [1, 2, 3, 4, 5, 6, 7] {
        let mut wrong = connect(&url).await;
        assert_eq!(
            authenticate(&mut wrong, &token, version).await,
            json!({"type":"incompatibleProtocol", "supportedVersion":8})
        );
    }

    let mut owner = connect(&url).await;
    assert_eq!(
        authenticate(&mut owner, &token, 8).await["type"],
        "authenticated"
    );
    let mut claimed_wrong_version = connect(&url).await;
    assert_eq!(
        authenticate(&mut claimed_wrong_version, &token, 2).await["type"],
        "incompatibleProtocol"
    );
    let mut duplicate = connect(&url).await;
    assert_eq!(
        authenticate(&mut duplicate, &token, 8).await["code"],
        "connectionInUse"
    );
    send_json(&mut owner, json!({"type":"createRoom", "sequence":1})).await;
    assert_eq!(next_json(&mut owner).await["type"], "admitted");
    owner.close(None).await.unwrap();

    tokio::task::yield_now().await;
    let mut consumed = connect(&url).await;
    assert_eq!(
        authenticate(&mut consumed, &token, 8).await["code"],
        "authenticationFailed"
    );
    server.abort();
}

#[tokio::test]
async fn malformed_and_binary_inputs_do_not_reflect_credentials() {
    let state = state();
    let token = issue_token(&state);
    let (url, server) = start_server(state).await;
    let mut malformed = connect(&url).await;
    malformed
        .send(Message::Text(format!("{{\"token\":\"{token}\"").into()))
        .await
        .unwrap();
    let response = next_json(&mut malformed).await;
    assert_eq!(response["code"], "invalidMessage");
    assert!(!response.to_string().contains(&token));

    let mut binary = connect(&url).await;
    binary
        .send(Message::Binary(vec![1, 2, 3].into()))
        .await
        .unwrap();
    assert_eq!(next_json(&mut binary).await["code"], "invalidMessage");

    let mut oversized = connect(&url).await;
    oversized
        .send(Message::Text(
            "x".repeat(grenadiers_core::protocol::MAX_CLIENT_MESSAGE_BYTES + 1)
                .into(),
        ))
        .await
        .unwrap();
    assert!(matches!(
        within(oversized.next()).await,
        Some(Ok(Message::Close(_))) | Some(Err(_)) | None
    ));
    server.abort();
}

#[tokio::test]
async fn control_frames_do_not_extend_authentication_deadline() {
    let state = state();
    let token = issue_token(&state);
    let (url, server) = start_server(state).await;
    let mut socket = connect(&url).await;
    for _ in 0..4 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        socket.send(Message::Ping(Vec::new().into())).await.unwrap();
        assert!(matches!(
            within(socket.next()).await,
            Some(Ok(Message::Pong(_)))
        ));
    }
    assert_eq!(next_json(&mut socket).await["code"], "notAuthenticated");

    let mut fresh = connect(&url).await;
    assert_eq!(
        authenticate(&mut fresh, &token, 8).await["type"],
        "authenticated"
    );
    server.abort();
}

#[tokio::test]
async fn game_upgrade_rejects_query_origin_and_shared_capacity() {
    let app_state = state();
    let (url, server) = start_server(app_state.clone()).await;
    assert_eq!(
        rejected(&format!("{url}?token=private"), &[ORIGIN]).await,
        axum::http::StatusCode::BAD_REQUEST
    );
    assert_eq!(rejected(&url, &[]).await, axum::http::StatusCode::FORBIDDEN);
    assert_eq!(
        rejected(&url, &[ORIGIN, ORIGIN]).await,
        axum::http::StatusCode::FORBIDDEN
    );
    server.abort();

    let saturated = state();
    let _permits = saturated
        .connections
        .clone()
        .acquire_many_owned(MAX_CONNECTIONS as u32)
        .await
        .unwrap();
    let (url, server) = start_server(saturated).await;
    assert_eq!(
        rejected(&url, &[ORIGIN]).await,
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
    server.abort();
}

#[tokio::test]
async fn diagnostic_and_authenticated_game_sockets_share_capacity_both_directions() {
    let app_state = state();
    let token = issue_token(&app_state);
    let _reserved = app_state
        .connections
        .clone()
        .acquire_many_owned((MAX_CONNECTIONS - 1) as u32)
        .await
        .unwrap();
    let (game_url, server) = start_server(app_state.clone()).await;
    let diagnostic_url = diagnostic_url(&game_url);

    let mut diagnostic = connect(&diagnostic_url).await;
    assert_eq!(next_json(&mut diagnostic).await["type"], "welcome");
    assert_eq!(
        rejected(&game_url, &[ORIGIN]).await,
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );

    diagnostic.close(None).await.unwrap();
    wait_for_available_connection(&app_state).await;
    let mut game = connect(&game_url).await;
    assert_eq!(
        authenticate(&mut game, &token, 8).await["type"],
        "authenticated"
    );
    assert_eq!(
        rejected(&diagnostic_url, &[ORIGIN]).await,
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );

    game.close(None).await.unwrap();
    wait_for_available_connection(&app_state).await;
    let mut replacement_diagnostic = connect(&diagnostic_url).await;
    assert_eq!(
        next_json(&mut replacement_diagnostic).await["type"],
        "welcome"
    );
    server.abort();
}

#[tokio::test]
async fn abrupt_authenticated_disconnect_releases_lease_and_connection_permit() {
    let app_state = state();
    let token = issue_token(&app_state);
    let (url, server) = start_server(app_state.clone()).await;
    let mut socket = connect(&url).await;
    assert_eq!(
        authenticate(&mut socket, &token, 8).await["type"],
        "authenticated"
    );
    assert_eq!(
        app_state.connections.available_permits(),
        MAX_CONNECTIONS - 1
    );

    drop(socket);
    within(async {
        while app_state.connections.available_permits() != MAX_CONNECTIONS {
            tokio::task::yield_now().await;
        }
    })
    .await;
    let mut retry = connect(&url).await;
    assert_eq!(
        authenticate(&mut retry, &token, 8).await["code"],
        "authenticationFailed"
    );
    server.abort();
}
