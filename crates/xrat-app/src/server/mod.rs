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

use crate::app::config::{RoutingSettings, ServerSettings};
use crate::app::events;
use xrat_db::Database;

pub use error::{ServerError, ServerResult};
pub use routes::pac::proxy_pac;
pub use state::ServerState;

mod listener;
pub use listener::build_router;
pub use listener::build_router_from_context;
pub use listener::parse_bind_addr_public;
pub use listener::serve;
pub use listener::serve_with_shutdown;
pub use listener::serve_with_signal;

mod web;
pub use web::hash_password;

async fn web_headers(
    axum::extract::State(state): axum::extract::State<ServerState>,
    request: Request,
    next: Next,
) -> Response {
    static REQUEST_SLOTS: std::sync::LazyLock<std::sync::Arc<tokio::sync::Semaphore>> =
        std::sync::LazyLock::new(|| std::sync::Arc::new(tokio::sync::Semaphore::new(128)));
    let Ok(_permit) = REQUEST_SLOTS.clone().try_acquire_owned() else {
        use axum::response::IntoResponse;
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    for (key, value) in [
        ("cache-control", "no-store"),
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        ("x-frame-options", "DENY"),
        (
            "content-security-policy",
            "default-src 'self'; script-src 'self' 'wasm-unsafe-eval' 'sha256-pTs8XU8t4UQbuv6Ly0IJtzFW9IQzRKWA/LBCdbqpT34='; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'",
        ),
    ] {
        headers.insert(
            axum::http::HeaderName::from_static(key),
            axum::http::HeaderValue::from_static(value),
        );
    }
    if state.web.as_ref().is_some_and(|web| web.secure) {
        response.headers_mut().insert(
            "strict-transport-security",
            axum::http::HeaderValue::from_static("max-age=31536000"),
        );
    }
    response
}

pub async fn serve_context(
    context: &crate::app::context::AppContext,
    settings: &ServerSettings,
    shutdown: Option<tokio::sync::oneshot::Receiver<()>>,
) -> crate::app::Result<()> {
    let mut state =
        ServerState::from_settings(context.db.clone(), settings, &context.app_config.routing)?;
    state.services = context.services();
    state.context = Some(context.clone());
    state.services = context.services();
    if let Some(web) = state.web.as_mut() {
        let web = std::sync::Arc::get_mut(web).expect("new server state is unshared");
        if context
            .db
            .web_users()
            .await?
            .iter()
            .any(|user| user.username == web.admin.username)
        {
            return Err(crate::app::AppError::InvalidArgument(
                "configured administrator conflicts with a database user".into(),
            ));
        }
        web.context = Some(context.clone());
    }
    if !context.is_machine_client()
        && settings.key.is_none()
        && !settings
            .host
            .parse::<IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
    {
        tracing::warn!("legacy subscription exports are public without server.key");
    }
    let router = build_router(state);
    listener::listen(
        router,
        settings,
        async move {
            if let Some(shutdown) = shutdown {
                let _ = shutdown.await;
            } else if let Err(error) = xrat_support::signals::wait_for_shutdown().await {
                tracing::warn!(%error, "shutdown signal failed");
            }
        },
        settings.web.enabled.then(|| context.db.clone()),
    )
    .await
}

mod tls;
