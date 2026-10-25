use std::{future::pending, io, time::Duration};

use tokio::{
    sync::oneshot,
    task::JoinHandle,
    time::{Instant, timeout},
};

use super::{AppState, ShutdownOutcome, run_with_shutdown};

const ORIGIN: &str = "http://localhost:5177";

fn state() -> AppState {
    AppState::without_database([ORIGIN.to_owned()])
}

async fn assert_deadline_completion(
    shutdown: JoinHandle<(Instant, io::Result<ShutdownOutcome>)>,
    signal_instant: Instant,
) {
    let (completed_at, outcome) = timeout(Duration::from_secs(1), shutdown)
        .await
        .expect("shutdown did not complete at its five-second deadline")
        .unwrap();
    assert_eq!(completed_at, signal_instant + Duration::from_secs(5));
    assert_eq!(outcome.unwrap(), ShutdownOutcome::DeadlineElapsed);
}

#[tokio::test(start_paused = true)]
async fn shutdown_deadline_starts_when_the_signal_is_observed() {
    let state = state();
    let (signal_tx, signal_rx) = oneshot::channel();
    let (observed_tx, observed_rx) = oneshot::channel();
    let shutdown = tokio::spawn(async move {
        let outcome = run_with_shutdown(
            state,
            async move {
                signal_rx.await.unwrap();
                observed_tx.send(Instant::now()).unwrap();
            },
            |_| pending::<io::Result<()>>(),
        )
        .await;
        (Instant::now(), outcome)
    });

    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(6)).await;
    assert!(!shutdown.is_finished());

    signal_tx.send(()).unwrap();
    let signal_instant = observed_rx.await.unwrap();
    tokio::time::advance(Duration::from_millis(4_999)).await;
    assert!(!shutdown.is_finished());
    tokio::time::advance(Duration::from_millis(1)).await;
    tokio::task::yield_now().await;
    assert_deadline_completion(shutdown, signal_instant).await;
}

async fn assert_shared_deadline_with_held_permit(hold_diagnostic: bool) {
    let state = state();
    let semaphore = if hold_diagnostic {
        state.diagnostic_connections.clone()
    } else {
        state.connections.clone()
    };
    let permit = semaphore.acquire_owned().await.unwrap();
    let (server_done_tx, server_done_rx) = oneshot::channel();
    let (observed_tx, observed_rx) = oneshot::channel();
    let shutdown = tokio::spawn(async move {
        let outcome = run_with_shutdown(
            state,
            async move {
                observed_tx.send(Instant::now()).unwrap();
            },
            |graceful| async move {
                graceful.await.unwrap();
                tokio::time::sleep(Duration::from_secs(4)).await;
                server_done_tx.send(()).unwrap();
                Ok(())
            },
        )
        .await;
        (Instant::now(), outcome)
    });

    let signal_instant = observed_rx.await.unwrap();
    tokio::time::advance(Duration::from_secs(4)).await;
    server_done_rx.await.unwrap();
    assert!(!shutdown.is_finished());
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    assert_deadline_completion(shutdown, signal_instant).await;
    drop(permit);
}

#[tokio::test(start_paused = true)]
async fn server_and_gameplay_drain_share_one_deadline() {
    assert_shared_deadline_with_held_permit(false).await;
}

#[tokio::test(start_paused = true)]
async fn server_and_diagnostic_drain_share_one_deadline() {
    assert_shared_deadline_with_held_permit(true).await;
}

#[tokio::test]
async fn server_failure_before_signal_is_returned_without_starting_shutdown() {
    let state = state();
    let observed = run_with_shutdown(state.clone(), pending(), |_| async {
        Err(io::Error::new(
            io::ErrorKind::AddrInUse,
            "early server failure",
        ))
    })
    .await
    .unwrap_err();

    assert_eq!(observed.kind(), io::ErrorKind::AddrInUse);
    assert_eq!(observed.to_string(), "early server failure");
    assert!(!state.game_rooms.is_stopping());
}

#[tokio::test]
async fn server_failure_after_signal_beats_a_hung_app_drain() {
    let state = state();
    let permit = state.connections.clone().acquire_owned().await.unwrap();
    let observed = run_with_shutdown(state, async {}, |graceful| async move {
        graceful.await.unwrap();
        Err(io::Error::new(
            io::ErrorKind::ConnectionAborted,
            "failure during drain",
        ))
    })
    .await
    .unwrap_err();

    assert_eq!(observed.kind(), io::ErrorKind::ConnectionAborted);
    assert_eq!(observed.to_string(), "failure during drain");
    drop(permit);
}
