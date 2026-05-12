use crate::app::daemon::server::RotationTrigger;
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeConnectResult,
    RuntimeDisconnectResult, RuntimeReplaceResult, RuntimeStatusResult, SupervisorState,
};
use crate::app::runtime::AppContext;
use tokio::sync::oneshot;

mod runtime_lifecycle;
mod runtime_status_connect;

pub(super) async fn handle_runtime_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeStatusResult>,
) {
    runtime_status_connect::handle_runtime_status(state, context, respond_to).await;
}

pub(super) async fn handle_runtime_connect(
    state: &SupervisorState,
    context: &AppContext,
    config_id: i64,
    respond_to: oneshot::Sender<RuntimeConnectResult>,
) {
    runtime_status_connect::handle_runtime_connect(state, context, config_id, respond_to).await;
}

pub(super) async fn handle_runtime_disconnect(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeDisconnectResult>,
) {
    runtime_lifecycle::handle_runtime_disconnect(state, context, respond_to).await;
}

pub(super) async fn handle_runtime_replace(
    state: &mut SupervisorState,
    context: &AppContext,
    trigger: RotationTrigger,
    candidate_id: Option<i64>,
    respond_to: oneshot::Sender<RuntimeReplaceResult>,
) {
    runtime_lifecycle::handle_runtime_replace(state, context, trigger, candidate_id, respond_to)
        .await;
}

pub(super) async fn handle_daemon_shutdown(
    context: &AppContext,
    respond_to: oneshot::Sender<DaemonShutdownResult>,
) {
    runtime_lifecycle::handle_daemon_shutdown(context, respond_to).await;
}

pub(super) async fn handle_proxy_start(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    runtime_lifecycle::handle_proxy_start(state, context, respond_to).await;
}

pub(super) async fn handle_proxy_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyStatusResult>,
) {
    runtime_lifecycle::handle_proxy_status(state, context, respond_to).await;
}

pub(super) async fn handle_proxy_stop(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    runtime_lifecycle::handle_proxy_stop(state, context, respond_to).await;
}
