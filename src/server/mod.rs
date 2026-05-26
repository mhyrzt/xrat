mod auth;
mod error;
mod response;
mod routes;
mod state;

use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;

use crate::app::config::ServerSettings;
use crate::db::Database;

pub use error::{ServerError, ServerResult};
pub use state::ServerState;

pub fn build_router(state: ServerState) -> Router {
    routes::router().with_state(state)
}

pub async fn serve(db: Database, settings: &ServerSettings) -> crate::app::Result<()> {
    let state = ServerState::from_settings(db, settings)?;
    let router = build_router(state);
    let addr: SocketAddr = format!("{}:{}", settings.host, settings.port)
        .parse()
        .map_err(|err| {
            crate::app::AppError::InvalidArgument(format!(
                "invalid server bind address {}:{}: {err}",
                settings.host, settings.port
            ))
        })?;
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "HTTP API server listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            if let Err(err) = tokio::signal::ctrl_c().await {
                tracing::warn!(error = %err, "failed to listen for shutdown signal");
            }
            tracing::info!("HTTP API server shutting down");
        })
        .await?;

    Ok(())
}
