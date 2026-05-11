use crate::app::daemon::server::{
    DaemonShutdownPayload, RotationTrigger, RuntimeDisconnectPayload, RuntimeReplacePayload,
};
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, RuntimeDisconnectResult, RuntimeReplaceResult, SupervisorState,
};
use crate::app::runtime::AppContext;
use crate::app::runtime_service::{ReplaceRequest, RuntimeService};
use tokio::sync::oneshot;

pub(super) async fn handle_runtime_disconnect(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeDisconnectResult>,
) {
    let active_session_id = context
        .db
        .get_running_runtime_session()
        .await
        .ok()
        .flatten()
        .map(|session| session.id);
    match RuntimeService::new(context).disconnect().await {
        Ok(result) => {
            if result.stopped_session
                && let Some(session_id) = active_session_id
            {
                let _ = context
                    .db
                    .update_runtime_session_transition_metadata(
                        session_id,
                        Some("daemon"),
                        Some(&state.instance_id),
                        Some("manual_disconnect"),
                        Some("daemon runtime disconnect request succeeded"),
                        Some("daemon"),
                    )
                    .await;
            }
            let _ = respond_to.send(RuntimeDisconnectResult::Ok(RuntimeDisconnectPayload {
                stopped_session: result.stopped_session,
            }));
        }
        Err(err) => {
            let _ = respond_to.send(RuntimeDisconnectResult::Err {
                message: err.to_string(),
            });
        }
    }
}

pub(super) async fn handle_runtime_replace(
    state: &SupervisorState,
    context: &AppContext,
    trigger: RotationTrigger,
    candidate_id: Option<i64>,
    respond_to: oneshot::Sender<RuntimeReplaceResult>,
) {
    match RuntimeService::new(context)
        .replace(ReplaceRequest {
            trigger,
            candidate_id,
        })
        .await
    {
        Ok(result) => {
            let _ = context
                .db
                .update_runtime_session_transition_metadata(
                    result.new_session_id,
                    Some("daemon"),
                    Some(&state.instance_id),
                    Some("replace_commit_success"),
                    Some("daemon replace handoff completed"),
                    Some("daemon"),
                )
                .await;
            let _ = respond_to.send(RuntimeReplaceResult::Ok(RuntimeReplacePayload {
                trigger,
                replaced: true,
                old_session_id: result.old_session_id,
                new_config_id: result.new_config_id,
                new_session_id: result.new_session_id,
                new_pid: result.new_pid,
            }));
        }
        Err(err) => {
            let _ = respond_to.send(RuntimeReplaceResult::Err {
                message: err.to_string(),
            });
        }
    }
}

pub(super) async fn handle_daemon_shutdown(
    context: &AppContext,
    respond_to: oneshot::Sender<DaemonShutdownResult>,
) {
    let runtime_disconnected = RuntimeService::new(context)
        .disconnect()
        .await
        .map(|result| result.stopped_session)
        .unwrap_or(false);
    let _ = respond_to.send(DaemonShutdownResult::Ok(DaemonShutdownPayload {
        daemon_ready: false,
        runtime_disconnected,
    }));
}
