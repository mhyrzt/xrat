use super::*;

pub(super) async fn handle_proxy_start(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    let active = async {
        let id = context
            .db
            .get_running_runtime_session()
            .await?
            .and_then(|session| session.config_id);
        match id {
            Some(id) => context.db.get_config_by_id(id).await,
            None => Ok(None),
        }
    }
    .await;
    match active {
        Ok(Some(config)) if config.is_xray_profile() => {
            let _ = respond_to.send(ProxyControlResult::Err { message: "Automatic rotation is unsupported for a complete Xray profile; disconnect it first".into() });
            return;
        }
        Err(error) => {
            let _ = respond_to.send(ProxyControlResult::Err {
                message: error.to_string(),
            });
            return;
        }
        _ => {}
    }
    state.rotation_enabled = true;
    state.cooldown_active = false;
    state.next_timer_epoch_secs = Some(now_epoch_seconds() + state.rotation_interval_secs);
    crate::app::services::rotation::RotationService::new(context, &state.instance_id)
        .enabled(state.rotation_interval_secs)
        .await;
    if respond_to
        .send(ProxyControlResult::Ok(ProxyControlPayload {
            rotation_enabled: true,
        }))
        .is_err()
    {
        tracing::debug!(
            operation = "proxy_start_response",
            error = "response receiver dropped",
            "supervisor response dropped"
        );
    }
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
        .unwrap_or_else(|error| {
            tracing::debug!(operation = "proxy_status_config", %error, "failed to load optional supervisor state");
            None
        })
        .map(|record| record.id);
    if respond_to
        .send(ProxyStatusResult::Ok(ProxyStatusPayload {
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
            health_failure_threshold: state.health_failure_threshold,
            consecutive_health_failures: state.consecutive_health_failures,
            health_probe_in_flight: state.health_probe_in_flight,
            last_health_check_epoch_secs: state.last_health_check_epoch_secs,
            last_health_error: state.last_health_error.clone(),
            pending_health_recovery: state.pending_health_recovery,
        }))
        .is_err()
    {
        tracing::debug!(
            operation = "proxy_status_response",
            error = "response receiver dropped",
            "supervisor response dropped"
        );
    }
}

pub(super) async fn handle_proxy_stop(
    state: &mut SupervisorState,
    context: &AppContext,
    respond_to: oneshot::Sender<ProxyControlResult>,
) {
    state.rotation_enabled = false;
    state.cooldown_active = false;
    state.next_timer_epoch_secs = None;
    crate::app::services::rotation::RotationService::new(context, &state.instance_id)
        .disabled()
        .await;
    if respond_to
        .send(ProxyControlResult::Ok(ProxyControlPayload {
            rotation_enabled: false,
        }))
        .is_err()
    {
        tracing::debug!(
            operation = "proxy_stop_response",
            error = "response receiver dropped",
            "supervisor response dropped"
        );
    }
}
