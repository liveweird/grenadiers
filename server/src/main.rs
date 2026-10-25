#![forbid(unsafe_code)]

use std::net::SocketAddr;

use grenadiers_server::{Config, ShutdownOutcome, app, run_with_shutdown};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = Config::from_environment()?;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(configuration.log_filter()))
        .init();

    for warning in configuration.warnings() {
        tracing::warn!(%warning, "startup configuration warning");
    }
    let (state, address, web_dist_dir) = configuration.into_parts();
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
