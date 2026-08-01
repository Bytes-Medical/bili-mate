//! Service entry point: typed configuration from the environment, privacy-
//! safe structured logging, graceful shutdown on SIGTERM (spec 10 container
//! contract).

use std::net::SocketAddr;

use bili_mate_api::{app, AppState, Config};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("configuration error: {message}");
            std::process::exit(78);
        }
    };
    let bind = config.bind_address.clone();
    let state = AppState::new(config);

    if state.pack().is_none() {
        // Startup safety fault: serve HTTP so liveness works, but readiness
        // stays false and every clinical route answers 503 (OPS-004).
        tracing::error!(target: "bili_mate_api::startup", "rule pack integrity verification failed; service will not become ready");
    }

    let listener = match tokio::net::TcpListener::bind(&bind).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("failed to bind {bind}: {e}");
            std::process::exit(71);
        }
    };
    tracing::info!(target: "bili_mate_api::startup", address = %bind, "listening");

    let service = app(state).into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, service)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.ok();
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!(target: "bili_mate_api::startup", "shutdown signal received; draining");
}
