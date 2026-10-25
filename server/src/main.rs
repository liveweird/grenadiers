#![forbid(unsafe_code)]

use std::{net::SocketAddr, path::PathBuf};

use grenadiers_server::{AppState, ShutdownOutcome, app, run_with_shutdown};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let state = AppState::from_environment()?;
    let address: SocketAddr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8085".to_owned())
        .parse()?;
    let web_dist_dir = std::env::var_os("WEB_DIST_DIR").map(PathBuf::from);
    let listener = TcpListener::bind(address).await?;
    tracing::info!(%address, "server listening");
    let router = app(state.clone(), web_dist_dir);
    let outcome = run_with_shutdown(state, shutdown_signal(), |shutdown| {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move {
            let _ = shutdown.await;
        })
    })
    .await?;
    if outcome == ShutdownOutcome::DeadlineElapsed {
        tracing::warn!("shutdown deadline elapsed; terminating runtime");
    }
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
