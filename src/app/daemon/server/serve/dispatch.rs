use tokio::sync::mpsc;

use crate::app::daemon::server::DaemonRequestKind;
use crate::app::daemon::server::bridge::{
    daemon_shutdown_response_via_supervisor, ping_response_via_supervisor,
    proxy_start_response_via_supervisor, proxy_status_response_via_supervisor,
    proxy_stop_response_via_supervisor, runtime_connect_response_via_supervisor,
    runtime_disconnect_response_via_supervisor, runtime_replace_response_via_supervisor,
    runtime_status_response_via_supervisor,
};
use crate::app::daemon::supervisor::SupervisorEvent;

pub(super) async fn dispatch_request(
    request: DaemonRequestKind,
    supervisor_tx: mpsc::Sender<SupervisorEvent>,
) -> crate::app::Result<(Vec<u8>, bool)> {
    let response = match request {
        DaemonRequestKind::DaemonPing => (
            serde_json::to_vec(&ping_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::RuntimeStatus => (
            serde_json::to_vec(&runtime_status_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::RuntimeConnect { config_id } => (
            serde_json::to_vec(
                &runtime_connect_response_via_supervisor(supervisor_tx, config_id).await?,
            )?,
            false,
        ),
        DaemonRequestKind::RuntimeDisconnect => (
            serde_json::to_vec(&runtime_disconnect_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::RuntimeReplace {
            trigger,
            candidate_id,
        } => (
            serde_json::to_vec(
                &runtime_replace_response_via_supervisor(supervisor_tx, trigger, candidate_id)
                    .await?,
            )?,
            false,
        ),
        DaemonRequestKind::DaemonShutdown => (
            serde_json::to_vec(&daemon_shutdown_response_via_supervisor(supervisor_tx).await?)?,
            true,
        ),
        DaemonRequestKind::ProxyStart => (
            serde_json::to_vec(&proxy_start_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::ProxyStatus => (
            serde_json::to_vec(&proxy_status_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
        DaemonRequestKind::ProxyStop => (
            serde_json::to_vec(&proxy_stop_response_via_supervisor(supervisor_tx).await?)?,
            false,
        ),
    };
    Ok(response)
}
