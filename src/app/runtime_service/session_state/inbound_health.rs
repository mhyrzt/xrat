use super::*;

pub(crate) fn runtime_status_label(
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

pub(crate) async fn check_runtime_inbounds(
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
