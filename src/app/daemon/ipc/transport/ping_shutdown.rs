use super::roundtrip;
use tokio::sync::mpsc;

use crate::app::daemon::ipc::{
    DaemonResponse, DaemonResponseCode, DaemonShutdownPayload, PROTOCOL_VERSION, PingPayload,
    daemon_shutdown_response, ping_response,
};
use crate::app::daemon::supervisor::{DaemonShutdownResult, SupervisorEvent};

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
