use crate::app::context::AppContext;
use crate::app::daemon::ipc::{RuntimeConnectPayload, RuntimeStatusPayload};
use crate::app::daemon::supervisor::{RuntimeConnectResult, RuntimeStatusResult, SupervisorState};
use crate::app::runtime_service::{ConnectRequest, RuntimeService};
use crate::app::services::runtime_transitions::RuntimeTransitionService;
use tokio::sync::oneshot;
use xrat_model::ConfigId;

pub(super) async fn handle_runtime_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeStatusResult>,
) {
    let server = &context.app_config.server;
    let http_api_enabled = server.enabled;
    let http_api_addr = if http_api_enabled {
        Some(format!("{}:{}", server.host, server.port))
    } else {
        None
    };

    match RuntimeService::new(context).status().await {
        Ok(snapshot) => {
            if respond_to
                .send(RuntimeStatusResult::Ok(RuntimeStatusPayload {
                    tun: Some(crate::app::services::tun::capture_state(
                        context,
                        &snapshot,
                        &xrat_support::readiness::RuntimeProcessPorts::default(),
                    )),
                    daemon_ready: state.ready,
                    runtime_owned: snapshot.session.is_some() && snapshot.pid_running,
                    runtime_status: snapshot.status.as_str().to_string(),
                    session_id: snapshot.session.as_ref().map(|session| session.id),
                    active_config_id: snapshot.active_config.as_ref().map(|config| config.id),
                    pid_running: snapshot.pid_running,
                    http_api_enabled,
                    http_api_addr,
                }))
                .is_err()
            {
                tracing::debug!("runtime status response receiver dropped");
            }
        }
        Err(err) => {
            if respond_to
                .send(RuntimeStatusResult::Err {
                    message: err.to_string(),
                })
                .is_err()
            {
                tracing::debug!("runtime status error receiver dropped");
            }
        }
    }
}

pub(super) async fn handle_runtime_connect(
    state: &mut SupervisorState,
    context: &AppContext,
    config_id: ConfigId,
    respond_to: oneshot::Sender<RuntimeConnectResult>,
) {
    match RuntimeTransitionService::new(context, &state.instance_id)
        .connect(ConnectRequest { config_id })
        .await
    {
        Ok(result) => {
            if result.config.is_xray_profile() {
                state.pause_rotation_for_xray_profile();
            }
            if respond_to
                .send(RuntimeConnectResult::Ok(RuntimeConnectPayload {
                    config_id: result.config.id,
                    session_id: result.session_id,
                    pid: result.pid,
                }))
                .is_err()
            {
                tracing::debug!(
                    config_id = config_id.0,
                    session_id = result.session_id,
                    "runtime connect response receiver dropped"
                );
            }
        }
        Err(err) => {
            if respond_to
                .send(RuntimeConnectResult::Err {
                    message: err.to_string(),
                })
                .is_err()
            {
                tracing::debug!(
                    config_id = config_id.0,
                    "runtime connect error receiver dropped"
                );
            }
        }
    }
}
