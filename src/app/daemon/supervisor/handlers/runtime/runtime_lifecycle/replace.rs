use super::*;
use crate::app::runtime_service::ReplaceRequest;

pub(super) async fn handle_runtime_replace(
    state: &mut SupervisorState,
    context: &AppContext,
    trigger: RotationTrigger,
    candidate_id: Option<i64>,
    respond_to: oneshot::Sender<RuntimeReplaceResult>,
) {
    let active_session = context
        .db
        .get_running_runtime_session()
        .await
        .ok()
        .flatten();
    if let Some(session) = &active_session {
        let started_reason = rotation_started_reason(trigger);
        let _ = context
            .db
            .update_runtime_session_transition_metadata(
                session.id,
                Some("daemon"),
                Some(&state.instance_id),
                Some(started_reason),
                Some("proxy rotation replacement requested"),
                Some("daemon"),
            )
            .await;
    }

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
            state.last_candidate_config_id = Some(result.new_config_id);
            state.last_candidate_result = "replace_commit_success".to_string();
            state.cooldown_active = false;
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
            let message = err.to_string();
            let failure_reason = rotation_failure_reason(&message);
            state.last_trigger = Some(trigger);
            state.last_result = failure_reason.to_string();
            state.last_candidate_config_id = candidate_id;
            state.last_candidate_result = failure_reason.to_string();
            state.cooldown_active = trigger == RotationTrigger::HealthCheckFailed;
            if let Some(session) = &active_session {
                let _ = context
                    .db
                    .update_runtime_session_transition_metadata(
                        session.id,
                        Some("daemon"),
                        Some(&state.instance_id),
                        Some(failure_reason),
                        Some(&message),
                        Some("daemon"),
                    )
                    .await;
            }
            let _ = respond_to.send(RuntimeReplaceResult::Err { message });
        }
    }
}
