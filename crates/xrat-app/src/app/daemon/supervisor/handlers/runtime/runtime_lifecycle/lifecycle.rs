use super::*;

pub(in super::super) async fn handle_runtime_disconnect(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<RuntimeDisconnectResult>,
) {
    disconnect::handle_runtime_disconnect(state, context, respond_to).await;
}

pub(in super::super) async fn handle_runtime_replace(
    state: &mut SupervisorState,
    context: &AppContext,
    trigger: RotationTrigger,
    candidate_id: Option<ConfigId>,
    respond_to: oneshot::Sender<RuntimeReplaceResult>,
) {
    replace::handle_runtime_replace(state, context, trigger, candidate_id, respond_to).await;
}

pub(in super::super) async fn handle_proxy_start(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    proxy::handle_proxy_start(state, context, respond_to).await;
}

pub(in super::super) async fn handle_proxy_status(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyStatusResult>,
) {
    proxy::handle_proxy_status(state, context, respond_to).await;
}

pub(in super::super) async fn handle_proxy_stop(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    proxy::handle_proxy_stop(state, context, respond_to).await;
}

pub(in super::super) async fn handle_daemon_shutdown(
    state: &SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<DaemonShutdownResult>,
) {
    let runtime_disconnected =
        crate::app::services::runtime_transitions::RuntimeTransitionService::shutdown(
            context,
            &state.instance_id,
        )
        .await;
    if respond_to
        .send(DaemonShutdownResult::Ok(DaemonShutdownPayload {
            daemon_ready: false,
            runtime_disconnected,
        }))
        .is_err()
    {
        tracing::debug!(
            operation = "daemon_shutdown_response",
            error = "response receiver dropped",
            "supervisor response dropped"
        );
    }
}
