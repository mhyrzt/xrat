use crate::app::context::AppContext;
use crate::app::daemon::ipc::{RuntimeConnectPayload, RuntimeStatusPayload};
use crate::app::daemon::supervisor::{RuntimeConnectResult, RuntimeStatusResult, SupervisorState};
use crate::app::runtime_service::{ConnectRequest, RuntimeService};
use tokio::sync::oneshot;

pub(super) async fn handle_runtime_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeStatusResult>,
) {
    match RuntimeService::new(context).status().await {
        Ok(snapshot) => {
            let _ = respond_to.send(RuntimeStatusResult::Ok(RuntimeStatusPayload {
                daemon_ready: state.ready,
                runtime_owned: snapshot.session.is_some() && snapshot.pid_running,
                runtime_status: snapshot.status.as_str().to_string(),
                session_id: snapshot.session.as_ref().map(|session| session.id),
                active_config_id: snapshot.active_config.as_ref().map(|config| config.id),
                pid_running: snapshot.pid_running,
            }));
        }
        Err(err) => {
            let _ = respond_to.send(RuntimeStatusResult::Err {
                message: err.to_string(),
            });
        }
    }
}

pub(super) async fn handle_runtime_connect(
    state: &SupervisorState,
    context: &AppContext,
    config_id: i64,
    respond_to: oneshot::Sender<RuntimeConnectResult>,
) {
    match RuntimeService::new(context)
        .connect(ConnectRequest { config_id })
        .await
    {
        Ok(result) => {
            let _ = context
                .db
                .update_runtime_session_transition_metadata(
                    result.session_id,
                    Some("daemon"),
                    Some(&state.instance_id),
                    Some("manual_connect"),
                    Some("daemon runtime connect request succeeded"),
                    Some("daemon"),
                )
                .await;
            let _ = respond_to.send(RuntimeConnectResult::Ok(RuntimeConnectPayload {
                config_id: result.config.id,
                session_id: result.session_id,
                pid: result.pid,
            }));
        }
        Err(err) => {
            let _ = respond_to.send(RuntimeConnectResult::Err {
                message: err.to_string(),
            });
        }
    }
}
