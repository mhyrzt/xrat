use tokio::sync::{mpsc, oneshot};

use crate::app::daemon::server::{
    DaemonResponse, DaemonResponseCode, DaemonShutdownPayload, PROTOCOL_VERSION, PingPayload,
    ProxyControlPayload, ProxyStatusPayload, RotationTrigger, RuntimeConnectPayload,
    RuntimeDisconnectPayload, RuntimeReplacePayload, RuntimeStatusPayload,
    daemon_shutdown_response, ping_response, proxy_control_error_response, proxy_control_response,
    proxy_status_error_response, proxy_status_response, runtime_connect_error_response,
    runtime_connect_response, runtime_disconnect_error_response, runtime_disconnect_response,
    runtime_replace_error_response, runtime_replace_response, runtime_status_error_response,
    runtime_status_response,
};
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeConnectResult,
    RuntimeDisconnectResult, RuntimeReplaceResult, RuntimeStatusResult, SupervisorEvent,
};

pub async fn runtime_replace_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
    trigger: RotationTrigger,
    candidate_id: Option<i64>,
) -> crate::app::Result<DaemonResponse<RuntimeReplacePayload>> {
    let payload = roundtrip(supervisor_tx, |respond_to| {
        SupervisorEvent::RuntimeReplace {
            trigger,
            candidate_id,
            respond_to,
        }
    })
    .await?;
    Ok(match payload {
        RuntimeReplaceResult::Ok(payload) => runtime_replace_response(payload),
        RuntimeReplaceResult::Err { message } => runtime_replace_error_response(message),
    })
}

pub async fn runtime_connect_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
    config_id: i64,
) -> crate::app::Result<DaemonResponse<RuntimeConnectPayload>> {
    let payload = roundtrip(supervisor_tx, |respond_to| {
        SupervisorEvent::RuntimeConnect {
            config_id,
            respond_to,
        }
    })
    .await?;
    Ok(match payload {
        RuntimeConnectResult::Ok(payload) => runtime_connect_response(payload),
        RuntimeConnectResult::Err { message } => runtime_connect_error_response(message),
    })
}

pub async fn runtime_disconnect_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<RuntimeDisconnectPayload>> {
    let payload = roundtrip(supervisor_tx, |respond_to| {
        SupervisorEvent::RuntimeDisconnect { respond_to }
    })
    .await?;
    Ok(match payload {
        RuntimeDisconnectResult::Ok(payload) => runtime_disconnect_response(payload),
        RuntimeDisconnectResult::Err { message } => runtime_disconnect_error_response(message),
    })
}

pub async fn runtime_status_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<RuntimeStatusPayload>> {
    let payload = roundtrip(supervisor_tx, |respond_to| SupervisorEvent::RuntimeStatus {
        respond_to,
    })
    .await?;
    Ok(match payload {
        RuntimeStatusResult::Ok(payload) => runtime_status_response(payload),
        RuntimeStatusResult::Err { message } => runtime_status_error_response(message),
    })
}

pub async fn ping_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<PingPayload>> {
    let payload = roundtrip(supervisor_tx, |respond_to| SupervisorEvent::DaemonPing {
        respond_to,
    })
    .await?;
    Ok(DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: ping_response().message,
        payload: Some(payload),
    })
}

pub async fn daemon_shutdown_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<DaemonShutdownPayload>> {
    let payload = roundtrip(supervisor_tx, |respond_to| {
        SupervisorEvent::DaemonShutdown { respond_to }
    })
    .await?;
    Ok(match payload {
        DaemonShutdownResult::Ok(payload) => daemon_shutdown_response(payload),
        DaemonShutdownResult::Err { message } => DaemonResponse {
            protocol_version: PROTOCOL_VERSION,
            ok: false,
            code: DaemonResponseCode::InvalidState,
            message,
            payload: None,
        },
    })
}

pub async fn proxy_start_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    let payload = roundtrip(supervisor_tx, |respond_to| SupervisorEvent::ProxyStart {
        respond_to,
    })
    .await?;
    Ok(match payload {
        ProxyControlResult::Ok(payload) => {
            proxy_control_response(payload, "proxy rotation scheduling enabled")
        }
        ProxyControlResult::Err { message } => proxy_control_error_response(message),
    })
}

pub async fn proxy_stop_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<ProxyControlPayload>> {
    let payload = roundtrip(supervisor_tx, |respond_to| SupervisorEvent::ProxyStop {
        respond_to,
    })
    .await?;
    Ok(match payload {
        ProxyControlResult::Ok(payload) => {
            proxy_control_response(payload, "proxy rotation scheduling disabled")
        }
        ProxyControlResult::Err { message } => proxy_control_error_response(message),
    })
}

pub async fn proxy_status_response_via_supervisor(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<DaemonResponse<ProxyStatusPayload>> {
    let payload = roundtrip(supervisor_tx, |respond_to| SupervisorEvent::ProxyStatus {
        respond_to,
    })
    .await?;
    Ok(match payload {
        ProxyStatusResult::Ok(payload) => proxy_status_response(payload),
        ProxyStatusResult::Err { message } => proxy_status_error_response(message),
    })
}

async fn roundtrip<T>(
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
    build_event: impl FnOnce(oneshot::Sender<T>) -> SupervisorEvent,
) -> crate::app::Result<T> {
    let (tx, rx) = oneshot::channel();
    supervisor_tx.send(build_event(tx)).await.map_err(|_| {
        crate::app::AppError::InvalidArgument("supervisor is not running".to_string())
    })?;
    rx.await.map_err(|_| {
        crate::app::AppError::InvalidArgument("supervisor response channel closed".to_string())
    })
}
