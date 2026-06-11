mod auth;
mod error;
mod response;
mod routes;
mod state;

#[cfg(test)]
mod tests;

use std::net::{IpAddr, SocketAddr};

use axum::Router;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use tokio::net::TcpListener;

use crate::app::config::{RoutingSettings, ServerSettings};
use crate::app::events;
use crate::db::Database;

pub use error::{ServerError, ServerResult};
pub use routes::pac::{PacEndpoints, PacRules, render_pac};
pub use state::ServerState;

pub fn build_router(state: ServerState) -> Router {
    routes::router()
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            record_request,
        ))
        .with_state(state)
}

/// Record each handled HTTP request as a best-effort `api` event so the TUI API
/// tab and `xrat logs` can show server activity. Recording happens on a detached
/// task and never affects the response.
async fn record_request(
    State(state): State<ServerState>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status();

    let db = state.db.clone();
    tokio::spawn(async move {
        events::record(
            &db,
            level_for_status(status),
            events::SOURCE_API,
            method.as_str(),
            format!("{method} {path} -> {}", status.as_u16()),
            None,
            None,
            None,
        )
        .await;
    });

    response
}

fn level_for_status(status: StatusCode) -> &'static str {
    if status.is_server_error() {
        events::LEVEL_ERROR
    } else if status.is_client_error() {
        events::LEVEL_WARN
    } else {
        events::LEVEL_INFO
    }
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
