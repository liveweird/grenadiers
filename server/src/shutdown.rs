use std::{future::Future, io};

use tokio::{
    sync::oneshot,
    time::{Instant, timeout_at},
};

use crate::{AppState, SHUTDOWN_DRAIN_TIMEOUT};

/// Result of waiting for the HTTP server and application state to drain.
#[derive(Debug, PartialEq, Eq)]
pub enum ShutdownOutcome {
    /// The server and application state finished before the shutdown deadline.
    Completed,
    /// The shared shutdown deadline elapsed while asynchronous work remained.
    ///
    /// Returning this outcome stops waiting; it does not abort connection tasks spawned by
    /// Axum when the caller keeps the Tokio runtime alive.
    DeadlineElapsed,
}

/// Runs a server until it finishes or a shutdown signal starts one shared asynchronous budget.
///
/// After the signal, the HTTP server and raw application drain run concurrently against the
/// same five-second deadline. The executable caller can then return to tear down its runtime.
pub async fn run_with_shutdown<S, F>(
    state: AppState,
    signal: F,
    make_server: impl FnOnce(oneshot::Receiver<()>) -> S,
) -> io::Result<ShutdownOutcome>
where
    S: IntoFuture<Output = io::Result<()>>,
    F: Future<Output = ()>,
{
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    let server = make_server(shutdown_receiver).into_future();
    tokio::pin!(server);

    tokio::select! {
        result = &mut server => return result.map(|()| ShutdownOutcome::Completed),
        () = signal => {}
    }

    let deadline = Instant::now() + SHUTDOWN_DRAIN_TIMEOUT;
    state.begin_shutdown();
    let _ = shutdown_sender.send(());

    let drain = async {
        state.drain_shutdown().await;
        Ok::<(), io::Error>(())
    };
    match timeout_at(deadline, async { tokio::try_join!(&mut server, drain) }).await {
        Ok(Ok(_)) => Ok(ShutdownOutcome::Completed),
        Ok(Err(error)) => Err(error),
        Err(_) => Ok(ShutdownOutcome::DeadlineElapsed),
    }
}
