use super::*;

pub(super) struct ResolvedLaunch {
    pub(super) binary_path: PathBuf,
    pub(super) config: crate::xray::XrayConfig,
    pub(super) ready_host: String,
    pub(super) ready_port: u16,
    pub(super) endpoints: RuntimeEndpoints,
}

pub(super) async fn active_session_state(
    context: &AppContext,
) -> crate::app::Result<ActiveSessionState> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        return Ok(ActiveSessionState::None);
    };

    if runtime_session_is_alive(&session) {
        return Ok(ActiveSessionState::Running(session));
    }

    mark_session_stale(context, &session).await?;
    Ok(ActiveSessionState::Stale(session))
}

pub(super) async fn stop_active_session(context: &AppContext) -> crate::app::Result<bool> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        context.db.clear_active_config().await?;
        return Ok(false);
    };

    context
        .db
        .update_runtime_session_state(
            session.id,
            RuntimeSessionStatus::Stopping,
            None,
            None,
            None,
            None,
        )
        .await?;

    if let Some(pid) = session.process_id {
        let outcome = xray_runtime::terminate_process_gracefully(pid, SHUTDOWN_TIMEOUT)?;
        tracing::info!(
            session_id = session.id,
            pid,
            outcome = ?outcome,
            "runtime process termination completed"
        );
    } else {
        tracing::warn!(
            session_id = session.id,
            "runtime session has no saved process id"
        );
    }

    context
        .db
        .mark_runtime_session_stopped(session.id, Some(&now_string()))
        .await?;
    context.db.clear_active_config().await?;
    Ok(true)
}

async fn mark_session_stale(
    context: &AppContext,
    session: &RuntimeSessionRecord,
) -> crate::app::Result<()> {
    let terminal_status = match session.status {
        RuntimeSessionStatus::Stopping => RuntimeSessionStatus::Stopped,
        RuntimeSessionStatus::Starting | RuntimeSessionStatus::Running => {
            RuntimeSessionStatus::Failed
        }
        RuntimeSessionStatus::Stopped | RuntimeSessionStatus::Failed => return Ok(()),
    };

    context
        .db
        .update_runtime_session_state(
            session.id,
            terminal_status,
            None,
            None,
            Some(&now_string()),
            Some(stale_session_reason(session)),
        )
        .await?;
    context.db.clear_active_config().await?;
    Ok(())
}

fn stale_session_reason(session: &RuntimeSessionRecord) -> &'static str {
    match session.status {
        RuntimeSessionStatus::Stopping => "runtime process disappeared while stopping",
        RuntimeSessionStatus::Starting | RuntimeSessionStatus::Running => {
            "runtime process is not running"
        }
        RuntimeSessionStatus::Stopped | RuntimeSessionStatus::Failed => "runtime session is closed",
    }
}

pub(super) fn runtime_session_is_alive(session: &RuntimeSessionRecord) -> bool {
    session
        .process_id
        .map(xray_runtime::process_is_running)
        .unwrap_or(false)
}

pub(super) fn runtime_status_label(
    latest: &Option<RuntimeSessionRecord>,
    active_state: &ActiveSessionState,
    pid_running: bool,
    inbound_health: &RuntimeInboundHealth,
) -> RuntimeStatusLabel {
    match latest {
        None => RuntimeStatusLabel::Stopped,
        Some(_) if matches!(active_state, ActiveSessionState::Stale(_)) => {
            RuntimeStatusLabel::StaleReconciled
        }
        Some(session)
            if matches!(session.status, RuntimeSessionStatus::Running) && !pid_running =>
        {
            RuntimeStatusLabel::Stale
        }
        Some(session)
            if matches!(session.status, RuntimeSessionStatus::Running)
                && inbound_health.has_unreachable_endpoint() =>
        {
            RuntimeStatusLabel::Degraded
        }
        Some(session) => RuntimeStatusLabel::Persisted(session.status.clone()),
    }
}

pub(super) async fn check_runtime_inbounds(
    session: &RuntimeSessionRecord,
    pid_running: bool,
) -> RuntimeInboundHealth {
    RuntimeInboundHealth {
        socks: check_runtime_inbound(
            session.socks_host.as_deref(),
            session.socks_port,
            pid_running,
        )
        .await,
        http: check_runtime_inbound(session.http_host.as_deref(), session.http_port, pid_running)
            .await,
        shadowsocks: check_runtime_inbound(
            session.shadowsocks_host.as_deref(),
            session.shadowsocks_port,
            pid_running,
        )
        .await,
    }
}

async fn check_runtime_inbound(
    host: Option<&str>,
    port: Option<i64>,
    pid_running: bool,
) -> Option<RuntimeEndpointHealth> {
    let endpoint = endpoint_from_parts(host, port)?;
    let state = if pid_running {
        let connect_host = connect_host_for_bind_host(&endpoint.host);
        match timeout(
            INBOUND_LIVENESS_TIMEOUT,
            TcpStream::connect((connect_host.as_str(), endpoint.port)),
        )
        .await
        {
            Ok(Ok(_)) => RuntimeEndpointState::Reachable,
            Ok(Err(_)) | Err(_) => RuntimeEndpointState::Unreachable,
        }
    } else {
        RuntimeEndpointState::NotChecked
    };

    Some(RuntimeEndpointHealth { endpoint, state })
}

fn endpoint_from_parts(host: Option<&str>, port: Option<i64>) -> Option<RuntimeEndpoint> {
    let port = u16::try_from(port?).ok()?;
    Some(RuntimeEndpoint {
        host: host.unwrap_or("unknown").to_string(),
        port,
    })
}
