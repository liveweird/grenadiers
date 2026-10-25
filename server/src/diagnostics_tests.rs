use std::{future::Future, time::Duration};

use axum::http::{StatusCode, header};
use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpListener, task::JoinHandle, time::timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{
        Error as WebSocketError, Message, client::IntoClientRequest,
        protocol::frame::coding::CloseCode,
    },
};

use super::*;

const ORIGIN: &str = "http://localhost:5177";
const TEST_TIMEOUT: Duration = Duration::from_secs(2);

type ClientSocket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

fn state() -> AppState {
    AppState::without_database([ORIGIN.to_owned()])
}

async fn within<T>(future: impl Future<Output = T>) -> T {
    timeout(TEST_TIMEOUT, future)
        .await
        .expect("operation exceeded test timeout")
}

async fn rejected_handshake(
    url: &str,
    origins: &[&str],
) -> tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>> {
    let mut request = url.into_client_request().unwrap();
    for origin in origins {
        request
            .headers_mut()
            .append(header::ORIGIN, origin.parse().unwrap());
    }
    match within(connect_async(request)).await {
        Err(WebSocketError::Http(response)) => *response,
        other => panic!("expected HTTP rejection, got {other:?}"),
    }
}

#[tokio::test]
async fn diagnostic_upgrade_rejects_duplicate_and_null_origins_without_caching() {
    let app_state = state();
    let (url, server) = start_server(app_state.clone()).await;
    for origins in [[ORIGIN, ORIGIN].as_slice(), ["null"].as_slice()] {
        let response = rejected_handshake(&url, origins).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "application/problem+json"
        );
    }
    assert_eq!(
        app_state.diagnostic_connections.available_permits(),
        MAX_DIAGNOSTIC_CONNECTIONS
    );
    let mut accepted = connect(&url).await;
    assert!(matches!(
        within(accepted.next()).await,
        Some(Ok(Message::Text(_)))
    ));
    accepted.close(None).await.unwrap();
    server.abort();
}

#[tokio::test]
async fn diagnostic_upgrade_rate_limit_returns_documented_retry_and_no_store() {
    let app_state = state();
    let (url, server) = start_server(app_state.clone()).await;
    let now = std::time::Instant::now();
    for _ in 0..32 {
        assert!(app_state.diagnostic_upgrade_limit.allow(now));
    }

    let response = rejected_handshake(&url, &[ORIGIN]).await;
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(response.headers()[header::RETRY_AFTER], "1");
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    server.abort();
}

#[tokio::test]
async fn exhausted_diagnostic_upgrade_budget_does_not_block_gameplay_upgrade() {
    let app_state = state();
    let (diagnostic_url, server) = start_server(app_state.clone()).await;
    let now = std::time::Instant::now();
    for _ in 0..32 {
        assert!(app_state.diagnostic_upgrade_limit.allow(now));
    }
    assert!(!app_state.diagnostic_upgrade_limit.allow(now));

    let gameplay_url = diagnostic_url.replace("/api/v1/ws", "/api/v1/game/ws");
    let mut request = gameplay_url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert(header::ORIGIN, ORIGIN.parse().unwrap());
    let (mut socket, response) = within(connect_async(request)).await.unwrap();
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    within(socket.close(None)).await.unwrap();
    server.abort();
}

#[tokio::test]
async fn rejected_origin_does_not_consume_diagnostic_upgrade_budget() {
    let app_state = state();
    let (url, server) = start_server(app_state.clone()).await;
    let now = std::time::Instant::now();
    for _ in 0..31 {
        assert!(app_state.diagnostic_upgrade_limit.allow(now));
    }

    for origins in [["https://attacker.invalid"].as_slice(), ["null"].as_slice()] {
        assert_eq!(
            rejected_handshake(&url, origins).await.status(),
            StatusCode::FORBIDDEN
        );
    }
    assert!(app_state.diagnostic_upgrade_limit.allow(now));
    assert!(!app_state.diagnostic_upgrade_limit.allow(now));
    server.abort();
}

async fn start_server(app_state: AppState) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app(app_state, None).into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap();
    });
    (format!("ws://{address}/api/v1/ws"), task)
}

async fn connect(url: &str) -> ClientSocket {
    connect_forwarded(url, None).await
}

async fn connect_forwarded(url: &str, forwarded: Option<&str>) -> ClientSocket {
    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert(header::ORIGIN, ORIGIN.parse().unwrap());
    if let Some(forwarded) = forwarded {
        request
            .headers_mut()
            .insert("x-forwarded-for", forwarded.parse().unwrap());
    }
    within(connect_async(request)).await.unwrap().0
}

#[tokio::test]
async fn peer_diagnostic_exhaustion_does_not_consume_gameplay_or_another_peer() {
    let mut app_state = state();
    app_state.trusted_proxies = std::sync::Arc::new(
        crate::peer_identity::TrustedProxyIps::parse(Some("127.0.0.1"))
            .expect("trusted test proxy"),
    );
    let (diagnostic_url, server) = start_server(app_state).await;
    let mut peer_a = Vec::new();
    for _ in 0..4 {
        peer_a.push(connect_forwarded(&diagnostic_url, Some("198.51.100.1")).await);
    }
    let denied = rejected_handshake_forwarded(&diagnostic_url, "198.51.100.1").await;
    assert_eq!(denied.status(), StatusCode::TOO_MANY_REQUESTS);
    tokio::time::sleep(Duration::from_millis(1_050)).await;
    let capacity = rejected_handshake_forwarded(&diagnostic_url, "198.51.100.1").await;
    assert_eq!(capacity.status(), StatusCode::SERVICE_UNAVAILABLE);

    let gameplay_url = diagnostic_url.replace("/api/v1/ws", "/api/v1/game/ws");
    let mut gameplay = connect_forwarded(&gameplay_url, Some("198.51.100.1")).await;
    gameplay.close(None).await.unwrap();
    let mut peer_b = connect_forwarded(&diagnostic_url, Some("198.51.100.2")).await;
    peer_b.close(None).await.unwrap();
    drop(peer_a);
    server.abort();
}

async fn rejected_handshake_forwarded(
    url: &str,
    forwarded: &str,
) -> tokio_tungstenite::tungstenite::http::Response<Option<Vec<u8>>> {
    let mut request = url.into_client_request().unwrap();
    request
        .headers_mut()
        .insert(header::ORIGIN, ORIGIN.parse().unwrap());
    request
        .headers_mut()
        .insert("x-forwarded-for", forwarded.parse().unwrap());
    match within(connect_async(request)).await {
        Err(WebSocketError::Http(response)) => *response,
        other => panic!("expected HTTP rejection, got {other:?}"),
    }
}

#[tokio::test]
async fn diagnostic_control_frame_flood_closes_and_returns_its_permit() {
    let app_state = state();
    let (url, server) = start_server(app_state.clone()).await;
    let mut socket = connect(&url).await;
    assert!(matches!(
        within(socket.next()).await,
        Some(Ok(Message::Text(_)))
    ));
    assert_eq!(
        app_state.diagnostic_connections.available_permits(),
        MAX_DIAGNOSTIC_CONNECTIONS - 1
    );

    for _ in 0..100 {
        if socket.send(Message::Pong(Vec::new().into())).await.is_err() {
            break;
        }
    }
    let close = within(async {
        loop {
            match socket.next().await {
                Some(Ok(Message::Close(frame))) => break frame,
                Some(Ok(_)) => {}
                Some(Err(_)) | None => break None,
            }
        }
    })
    .await;
    assert_eq!(close.unwrap().code, CloseCode::Policy);

    within(async {
        while app_state.diagnostic_connections.available_permits() != MAX_DIAGNOSTIC_CONNECTIONS {
            tokio::task::yield_now().await;
        }
    })
    .await;
    server.abort();
}
