use super::*;

pub(super) async fn handle_proxy_start(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    state.rotation_enabled = true;
    state.cooldown_active = false;
    state.next_timer_epoch_secs = Some(now_epoch_seconds() + state.rotation_interval_secs);
    crate::app::events::record(
        &context.db,
        crate::app::events::LEVEL_INFO,
        crate::app::events::SOURCE_ROTATION,
        "rotation_enabled",
        format!(
            "Auto-rotation enabled (interval {}s)",
            state.rotation_interval_secs
        ),
        None,
        None,
        None,
    )
    .await;
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
        last_candidate_config_id: state.last_candidate_config_id,
        last_candidate_result: state.last_candidate_result.clone(),
        cooldown_active: state.cooldown_active,
        next_timer_epoch_secs: state.next_timer_epoch_secs,
    }));
}

pub(super) async fn handle_proxy_stop(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    state.rotation_enabled = false;
    state.cooldown_active = false;
    state.next_timer_epoch_secs = None;
    crate::app::events::record(
        &context.db,
        crate::app::events::LEVEL_INFO,
        crate::app::events::SOURCE_ROTATION,
        "rotation_disabled",
        "Auto-rotation disabled",
        None,
        None,
        None,
    )
    .await;
    let _ = respond_to.send(ProxyControlResult::Ok(ProxyControlPayload {
        rotation_enabled: false,
    }));
}
