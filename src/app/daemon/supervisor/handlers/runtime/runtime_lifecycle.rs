use crate::app::daemon::server::{
    DaemonShutdownPayload, ProxyControlPayload, ProxyStatusPayload, RotationTrigger,
    RuntimeDisconnectPayload, RuntimeReplacePayload,
};
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeDisconnectResult,
    RuntimeReplaceResult, SupervisorState,
};
use crate::app::runtime::AppContext;
use crate::app::runtime_service::{ReplaceRequest, RuntimeService};
use std::time::{SystemTime, UNIX_EPOCH};
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
    state: &mut SupervisorState,
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
            state.last_trigger = Some(trigger);
            state.last_result = "replace_commit_success".to_string();
            if state.rotation_enabled {
                state.next_timer_epoch_secs =
                    Some(now_epoch_seconds() + state.rotation_interval_secs);
            }
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
            state.last_trigger = Some(trigger);
            state.last_result = "replace_failed".to_string();
            let _ = respond_to.send(RuntimeReplaceResult::Err {
                message: err.to_string(),
            });
        }
    }
}

pub(super) async fn handle_proxy_start(
    state: &mut SupervisorState,
    _context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    state.rotation_enabled = true;
    state.next_timer_epoch_secs = Some(now_epoch_seconds() + state.rotation_interval_secs);
    let _ = respond_to.send(ProxyControlResult::Ok(ProxyControlPayload {
        rotation_enabled: true,
    }));
}

pub(super) async fn handle_proxy_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyStatusResult>,
) {
    let active_config_id = context
        .db
        .get_active_config()
        .await
        .ok()
        .flatten()
        .map(|record| record.id);
    let _ = respond_to.send(ProxyStatusResult::Ok(ProxyStatusPayload {
        daemon_ready: state.ready,
        rotation_enabled: state.rotation_enabled,
        interval_secs: state.rotation_interval_secs,
        health_trigger_enabled: state.health_trigger_enabled,
        cooldown_secs: state.cooldown_secs,
        active_config_id,
        last_trigger: state.last_trigger,
        last_result: state.last_result.clone(),
        next_timer_epoch_secs: state.next_timer_epoch_secs,
    }));
}

pub(super) async fn handle_proxy_stop(
    state: &mut SupervisorState,
    _context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    state.rotation_enabled = false;
    state.next_timer_epoch_secs = None;
    let _ = respond_to.send(ProxyControlResult::Ok(ProxyControlPayload {
        rotation_enabled: false,
    }));
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
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
