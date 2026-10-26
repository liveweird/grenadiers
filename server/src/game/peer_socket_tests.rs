use super::*;

#[tokio::test]
async fn real_listener_enforces_proxy_identity_and_peer_fairness() {
    let mut app_state = state();
    app_state.trusted_proxies = std::sync::Arc::new(
        crate::peer_identity::TrustedProxyIps::parse(Some("127.0.0.1"))
            .expect("trusted test proxy"),
    );
    let (url, server) = start_server(app_state).await;
    assert_eq!(
        rejected_forwarded(&url, None).await,
        axum::http::StatusCode::BAD_REQUEST
    );

    let mut peer_a = Vec::new();
    for _ in 0..8 {
        peer_a.push(connect_with_forwarded(&url, Some("198.51.100.1")).await);
    }
    assert_eq!(
        rejected_forwarded(&url, Some("198.51.100.1")).await,
        axum::http::StatusCode::TOO_MANY_REQUESTS
    );
    tokio::time::sleep(Duration::from_millis(550)).await;
    assert_eq!(
        rejected_forwarded(&url, Some("198.51.100.1")).await,
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
    let mut peer_b = connect_with_forwarded(&url, Some("198.51.100.2")).await;
    peer_b.close(None).await.unwrap();
    drop(peer_a);
    server.abort();
}

#[tokio::test]
async fn real_listener_ignores_spoofed_forwarding_from_untrusted_peers() {
    let (url, server) = start_server(state()).await;
    let mut sockets = Vec::new();
    for index in 0..8 {
        let forwarded = format!("203.0.113.{}", index + 1);
        sockets.push(connect_with_forwarded(&url, Some(&forwarded)).await);
    }
    assert_eq!(
        rejected_forwarded(&url, Some("203.0.113.250")).await,
        axum::http::StatusCode::TOO_MANY_REQUESTS
    );
    drop(sockets);
    server.abort();
}

#[tokio::test]
async fn failed_authentication_releases_the_peer_socket_permit() {
    let mut app_state = state();
    app_state.trusted_proxies = std::sync::Arc::new(
        crate::peer_identity::TrustedProxyIps::parse(Some("127.0.0.1"))
            .expect("trusted test proxy"),
    );
    let (url, server) = start_server(app_state).await;
    let mut sockets = Vec::new();
    for _ in 0..8 {
        sockets.push(connect_with_forwarded(&url, Some("198.51.100.10")).await);
    }
    let mut failed = sockets.pop().expect("eight sockets");
    assert_eq!(
        authenticate(&mut failed, &"0".repeat(64), 12).await["code"],
        "authenticationFailed"
    );
    let _ = within(failed.next()).await;
    tokio::time::sleep(Duration::from_millis(550)).await;

    let replacement = connect_with_forwarded(&url, Some("198.51.100.10")).await;
    drop(replacement);
    drop(sockets);
    server.abort();
}
