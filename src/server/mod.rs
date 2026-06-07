mod auth;
mod error;
mod response;
mod routes;
mod state;

#[cfg(test)]
mod tests;

use std::net::{IpAddr, SocketAddr};

use axum::Router;
use tokio::net::TcpListener;

use crate::app::config::{RoutingSettings, ServerSettings};
use crate::db::Database;

pub use error::{ServerError, ServerResult};
pub use routes::pac::{PacEndpoints, PacRules, render_pac};
pub use state::ServerState;

pub fn build_router(state: ServerState) -> Router {
    routes::router().with_state(state)
}

pub async fn serve(
    db: Database,
    settings: &ServerSettings,
    routing: &RoutingSettings,
) -> crate::app::Result<()> {
    let state = ServerState::from_settings(db, settings, routing)?;
    let addr = parse_bind_addr(&settings.host, settings.port)?;
    let router = build_router(state);
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

pub async fn serve_with_shutdown(
    db: Database,
    settings: &ServerSettings,
    routing: &RoutingSettings,
    shutdown: tokio::sync::oneshot::Receiver<()>,
) -> crate::app::Result<()> {
    let state = ServerState::from_settings(db, settings, routing)?;
    let addr = parse_bind_addr(&settings.host, settings.port)?;
    let router = build_router(state);
    let listener = TcpListener::bind(addr).await?;

    tracing::info!(%addr, "HTTP API server listening (daemon-managed)");
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = shutdown.await;
            tracing::info!("HTTP API server shutting down (daemon-managed)");
        })
        .await?;

    Ok(())
}

pub fn parse_bind_addr_public(host: &str, port: u16) -> crate::app::Result<SocketAddr> {
    parse_bind_addr(host, port)
}

fn parse_bind_addr(host: &str, port: u16) -> crate::app::Result<SocketAddr> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(SocketAddr::new(ip, port));
    }

    format!("{host}:{port}").parse().map_err(|err| {
        crate::app::AppError::InvalidArgument(format!(
            "invalid server bind address {host}:{port}: {err}"
        ))
    })
}
