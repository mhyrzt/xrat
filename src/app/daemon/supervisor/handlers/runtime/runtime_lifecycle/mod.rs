use crate::app::daemon::server::{
    DaemonShutdownPayload, ProxyControlPayload, ProxyStatusPayload, RotationTrigger,
    RuntimeDisconnectPayload, RuntimeReplacePayload,
};
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeDisconnectResult,
    RuntimeReplaceResult, SupervisorState,
};
use crate::app::runtime::AppContext;
use crate::app::runtime_service::RuntimeService;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

mod disconnect;
mod proxy;
mod replace;

pub(super) async fn handle_runtime_disconnect(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeDisconnectResult>,
) {
    disconnect::handle_runtime_disconnect(state, context, respond_to).await;
}

pub(super) async fn handle_runtime_replace(
    state: &mut SupervisorState,
    context: &AppContext,
    trigger: RotationTrigger,
    candidate_id: Option<i64>,
    respond_to: oneshot::Sender<RuntimeReplaceResult>,
) {
    replace::handle_runtime_replace(state, context, trigger, candidate_id, respond_to).await;
}

pub(super) async fn handle_proxy_start(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    proxy::handle_proxy_start(state, context, respond_to).await;
}

pub(super) async fn handle_proxy_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyStatusResult>,
) {
    proxy::handle_proxy_status(state, context, respond_to).await;
}

pub(super) async fn handle_proxy_stop(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    proxy::handle_proxy_stop(state, context, respond_to).await;
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

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn rotation_started_reason(trigger: RotationTrigger) -> &'static str {
    match trigger {
        RotationTrigger::Manual => "rotation_manual_started",
        RotationTrigger::Timer => "rotation_timer_started",
        RotationTrigger::HealthCheckFailed => "rotation_health_started",
    }
}

fn rotation_failure_reason(message: &str) -> &'static str {
    if message.contains("no eligible replacement candidate") {
        "rotation_no_candidate"
    } else {
        "rotation_candidate_failed"
    }
}
