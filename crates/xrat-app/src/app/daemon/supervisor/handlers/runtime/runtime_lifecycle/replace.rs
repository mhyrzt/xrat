use super::*;
use crate::app::runtime_service::ReplaceRequest;
use crate::app::services::rotation::RotationService;

pub(super) async fn handle_runtime_replace(
    state: &mut SupervisorState,
    context: &AppContext,
    trigger: RotationTrigger,
    candidate_id: Option<ConfigId>,
    respond_to: oneshot::Sender<RuntimeReplaceResult>,
) {
    match RotationService::new(context, &state.instance_id)
        .replace(ReplaceRequest {
            trigger,
            candidate_id,
        })
        .await
    {
        Ok(result) => {
            state.last_trigger = Some(trigger);
            state.last_result = "replace_commit_success".to_string();
            state.last_candidate_config_id = Some(result.new_config_id);
            state.last_candidate_result = "replace_commit_success".to_string();
            state.cooldown_active = false;
            state.pending_health_recovery = false;
            state.consecutive_health_failures = 0;
            if context
                .db
                .get_config_by_id(result.new_config_id)
                .await
                .ok()
                .flatten()
                .is_some_and(|config| config.is_xray_profile())
            {
                state.pause_rotation_for_xray_profile();
            }
            if state.rotation_enabled {
                state.next_timer_epoch_secs =
                    Some(now_epoch_seconds() + state.rotation_interval_secs);
            }
            if respond_to
                .send(RuntimeReplaceResult::Ok(RuntimeReplacePayload {
                    trigger,
                    replaced: result.old_session_id.is_some(),
                    old_session_id: result.old_session_id,
                    new_config_id: result.new_config_id,
                    new_session_id: result.new_session_id,
                    new_pid: result.new_pid,
                }))
                .is_err()
            {
                tracing::debug!(
                    operation = "runtime_replace_response",
                    ?trigger,
                    ?candidate_id,
                    error = "response receiver dropped",
                    "supervisor response dropped"
                );
            }
        }
        Err(err) => {
            let message = err.message;
            let failure_reason = err.reason.as_str();
            state.last_trigger = Some(trigger);
            state.last_result = failure_reason.to_string();
            state.last_candidate_config_id = candidate_id;
            state.last_candidate_result = failure_reason.to_string();
            state.cooldown_active = trigger == RotationTrigger::HealthCheckFailed;
            state.pending_health_recovery = trigger == RotationTrigger::HealthCheckFailed;
            if state.rotation_enabled {
                state.next_timer_epoch_secs =
                    Some(now_epoch_seconds() + state.rotation_interval_secs);
            }
            if respond_to
                .send(RuntimeReplaceResult::Err { message })
                .is_err()
            {
                tracing::debug!(
                    operation = "runtime_replace_error_response",
                    ?trigger,
                    ?candidate_id,
                    error = "response receiver dropped",
                    "supervisor response dropped"
                );
            }
        }
    }
}
