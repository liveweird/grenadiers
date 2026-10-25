use std::time::Duration;

use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
    sync::oneshot,
    time::{Instant, timeout},
};

use super::{AppState, ShutdownOutcome, app, run_with_shutdown};
use crate::shutdown_listener_test_support::ReadAcknowledgingListener;

const ORIGIN: &str = "http://localhost:5177";

#[tokio::test(flavor = "current_thread")]
async fn partial_http_headers_cannot_extend_the_global_shutdown_deadline() {
    let tcp_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = tcp_listener.local_addr().unwrap();
    let partial_request = b"GET /api/v1/health HTTP/1.1\r\nHost: localhost\r\n";
    let (read_tx, read_rx) = oneshot::channel();
    let (dropped_tx, dropped_rx) = oneshot::channel();
    let listener =
        ReadAcknowledgingListener::new(tcp_listener, partial_request.len(), read_tx, dropped_tx);
    let state = AppState::without_database([ORIGIN.to_owned()]);
    let (signal_tx, signal_rx) = oneshot::channel();
    let (observed_tx, observed_rx) = oneshot::channel();
    let coordinator = tokio::spawn(async move {
        let outcome = run_with_shutdown(
            state.clone(),
            async move {
                signal_rx.await.unwrap();
                observed_tx.send(Instant::now()).unwrap();
            },
            move |graceful| {
                axum::serve(listener, app(state, None)).with_graceful_shutdown(async move {
                    let _ = graceful.await;
                })
            },
        )
        .await;
        (Instant::now(), outcome)
    });

    let mut client = timeout(Duration::from_secs(2), TcpStream::connect(address))
        .await
        .expect("client did not connect")
        .unwrap();
    timeout(Duration::from_secs(2), client.write_all(partial_request))
        .await
        .expect("client did not write partial headers")
        .unwrap();
    let consumed_bytes = timeout(Duration::from_secs(2), read_rx)
        .await
        .expect("Axum did not consume the partial headers")
        .unwrap();
    assert!(consumed_bytes >= partial_request.len());

    tokio::time::pause();
    signal_tx.send(()).unwrap();
    let signal_instant = observed_rx.await.unwrap();
    tokio::time::advance(Duration::from_millis(4_999)).await;
    assert!(!coordinator.is_finished());
    tokio::time::advance(Duration::from_millis(1)).await;
    tokio::task::yield_now().await;
    let (completed_at, outcome) = timeout(Duration::from_secs(1), coordinator)
        .await
        .expect("coordinator did not complete at its five-second deadline")
        .unwrap();
    let elapsed = completed_at.duration_since(signal_instant);
    // Tokio's pinned timer driver has one-millisecond resolution. Since real I/O runs before
    // this test pauses time, the clock may be between driver ticks and round up by one tick.
    assert!(elapsed >= Duration::from_secs(5));
    assert!(elapsed <= Duration::from_millis(5_001));
    assert_eq!(outcome.unwrap(), ShutdownOutcome::DeadlineElapsed);

    tokio::time::resume();
    drop(client);
    timeout(Duration::from_secs(1), dropped_rx)
        .await
        .expect("Axum connection task did not release the test stream")
        .unwrap();
}
