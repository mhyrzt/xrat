use super::roundtrip;
use tokio::sync::mpsc;

use crate::app::daemon::server::{
    DaemonResponse, ProxyControlPayload, ProxyStatusPayload, proxy_control_error_response,
    proxy_control_response, proxy_status_error_response, proxy_status_response,
};
use crate::app::daemon::supervisor::{ProxyControlResult, ProxyStatusResult, SupervisorEvent};

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
