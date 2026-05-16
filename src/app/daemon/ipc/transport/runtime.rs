use super::roundtrip;
use tokio::sync::mpsc;

use crate::app::daemon::ipc::{
    DaemonResponse, RotationTrigger, RuntimeConnectPayload, RuntimeDisconnectPayload,
    RuntimeReplacePayload, RuntimeStatusPayload, runtime_connect_error_response,
    runtime_connect_response, runtime_disconnect_error_response, runtime_disconnect_response,
    runtime_replace_error_response, runtime_replace_response, runtime_status_error_response,
    runtime_status_response,
};
use crate::app::daemon::supervisor::{
    RuntimeConnectResult, RuntimeDisconnectResult, RuntimeReplaceResult, RuntimeStatusResult,
    SupervisorEvent,
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
