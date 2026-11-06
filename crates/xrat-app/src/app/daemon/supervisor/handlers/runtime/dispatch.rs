use crate::app::context::AppContext;
use crate::app::daemon::ipc::RotationTrigger;
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeConnectResult,
    RuntimeDisconnectResult, RuntimeReplaceResult, RuntimeStatusResult, SupervisorState,
};
use tokio::sync::oneshot;
use xrat_model::ConfigId;

use super::{runtime_lifecycle, runtime_status_connect};
pub(crate) async fn handle_runtime_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeStatusResult>,
) {
    runtime_status_connect::handle_runtime_status(state, context, respond_to).await;
}

pub(crate) async fn handle_runtime_connect(
    state: &mut SupervisorState,
    context: &AppContext,
    config_id: ConfigId,
    respond_to: oneshot::Sender<RuntimeConnectResult>,
) {
    runtime_status_connect::handle_runtime_connect(state, context, config_id, respond_to).await;
}

pub(crate) async fn handle_runtime_disconnect(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeDisconnectResult>,
) {
    runtime_lifecycle::handle_runtime_disconnect(state, context, respond_to).await;
}

pub(crate) async fn handle_runtime_replace(
    state: &mut SupervisorState,
    context: &AppContext,
    trigger: RotationTrigger,
    candidate_id: Option<ConfigId>,
    respond_to: oneshot::Sender<RuntimeReplaceResult>,
) {
    runtime_lifecycle::handle_runtime_replace(state, context, trigger, candidate_id, respond_to)
        .await;
}

pub(crate) async fn handle_daemon_shutdown(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<DaemonShutdownResult>,
) {
    runtime_lifecycle::handle_daemon_shutdown(state, context, respond_to).await;
}

pub(crate) async fn handle_proxy_start(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    runtime_lifecycle::handle_proxy_start(state, context, respond_to).await;
}

pub(crate) async fn handle_proxy_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyStatusResult>,
) {
    runtime_lifecycle::handle_proxy_status(state, context, respond_to).await;
}

pub(crate) async fn handle_proxy_stop(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    runtime_lifecycle::handle_proxy_stop(state, context, respond_to).await;
}
