use super::*;

pub fn build_router(state: ServerState) -> Router {
    if let Some(web) = &state.web {
        super::web::traffic::start(web);
    }
    let mut router = routes::router();
    if state.web.is_some() {
        router = router.merge(super::web::router(state.clone()));
    }
    router
        .layer(axum::extract::DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            super::web_headers,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            record_request,
        ))
        .with_state(state)
}

/// Build a router from an application context.
///
/// Convenience seam for hosts that already hold an [`AppContext`]; the HTTP
/// server itself still builds its state from server and routing settings.
pub fn build_router_from_context(
    context: &crate::app::context::AppContext,
    settings: &ServerSettings,
    routing: &RoutingSettings,
) -> crate::app::Result<Router> {
    let mut state = ServerState::from_settings(context.db.clone(), settings, routing)?;
    state.services = context.services();
    state.context = Some(context.clone());
    state.services = context.services();
    if let Some(web) = state.web.as_mut() {
        std::sync::Arc::get_mut(web)
            .expect("new server state is unshared")
            .context = Some(context.clone());
    }
    Ok(build_router(state))
}

/// Record each handled HTTP request as a best-effort `api` event so the TUI API
/// tab and `xrat logs` can show server activity. Recording happens on a detached
/// task and never affects the response.
pub(super) async fn record_request(
    State(state): State<ServerState>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status();

    static EVENT_SLOTS: std::sync::LazyLock<std::sync::Arc<tokio::sync::Semaphore>> =
        std::sync::LazyLock::new(|| std::sync::Arc::new(tokio::sync::Semaphore::new(128)));
    let Ok(permit) = EVENT_SLOTS.clone().try_acquire_owned() else {
        tracing::warn!("dropping API event: event writer is saturated");
        return response;
    };
    let db = state.db.clone();
    tokio::spawn(async move {
        let _permit = permit;
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

pub(super) fn level_for_status(status: StatusCode) -> &'static str {
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
    serve_with_signal(
        db,
        settings,
        routing,
        std::sync::Arc::new(xrat_support::signals::CtrlCShutdown),
    )
    .await
}

pub async fn serve_with_signal(
    db: Database,
    settings: &ServerSettings,
    routing: &RoutingSettings,
    signal: std::sync::Arc<dyn xrat_support::signals::ShutdownSignal>,
) -> crate::app::Result<()> {
    let state = ServerState::from_settings(db, settings, routing)?;
    let addr = parse_bind_addr(&settings.host, settings.port)?;
    let router = build_router(state);
    let _ = addr;
    listen(
        router,
        settings,
        async move {
            if let Err(err) = signal.wait().await {
                tracing::warn!(error = %err, "failed to listen for shutdown signal");
            }
        },
        None,
    )
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
    let _ = addr;
    listen(
        router,
        settings,
        async {
            let _ = shutdown.await;
        },
        None,
    )
    .await?;

    Ok(())
}

pub fn parse_bind_addr_public(host: &str, port: u16) -> crate::app::Result<SocketAddr> {
    parse_bind_addr(host, port)
}

pub(super) fn parse_bind_addr(host: &str, port: u16) -> crate::app::Result<SocketAddr> {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(SocketAddr::new(ip, port));
    }

    format!("{host}:{port}").parse().map_err(|err| {
        crate::app::AppError::InvalidArgument(format!(
            "invalid server bind address {host}:{port}: {err}"
        ))
    })
}

pub(super) async fn listen(
    router: Router,
    settings: &ServerSettings,
    shutdown: impl std::future::Future<Output = ()> + Send + 'static,
    recovery_db: Option<Database>,
) -> crate::app::Result<()> {
    let addr = parse_bind_addr(&settings.host, settings.port)?;
    let listener = std::net::TcpListener::bind(addr)?;
    listener.set_nonblocking(true)?;
    let tls = if settings.web.transport == crate::app::config::WebTransport::DirectHttps {
        let certificate = settings.web.certificate.as_ref().ok_or_else(|| {
            crate::app::AppError::InvalidArgument("direct_https requires certificate".into())
        })?;
        let key = settings.web.private_key.as_ref().ok_or_else(|| {
            crate::app::AppError::InvalidArgument("direct_https requires private_key".into())
        })?;
        Some(super::tls::load(certificate, key).await?)
    } else {
        None
    };
    if let Some(db) = recovery_db {
        db.web_interrupt_jobs(super::web::security_now()).await?;
    }
    tracing::info!(%addr, "XRAT web server listening");
    let handle = axum_server::Handle::new();
    let shutdown_handle = handle.clone();
    let task = tokio::spawn(async move {
        shutdown.await;
        shutdown_handle.graceful_shutdown(Some(std::time::Duration::from_secs(10)));
    });
    let result = if let Some(tls) = tls {
        axum_server::from_tcp_rustls(listener, tls)?
            .handle(handle)
            .serve(router.into_make_service())
            .await
    } else {
        axum_server::from_tcp(listener)?
            .handle(handle)
            .serve(router.into_make_service())
            .await
    };
    task.abort();
    result?;
    Ok(())
}
