use crate::app::context::AppContext;
use crate::app::daemon::supervisor::SupervisorState;
use crate::app::runtime_service::RuntimeService;
use crate::support::time::now_epoch_seconds;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct HealthTickOutcome {
    pub health_failure_recorded: bool,
    pub timer_due: bool,
    pub cooldown_active: bool,
}

pub(super) async fn handle_health_tick(
    state: &SupervisorState,
    context: &AppContext,
) -> HealthTickOutcome {
    let now = now_epoch_seconds();
    let timer_due = state.rotation_enabled
        && state
            .next_timer_epoch_secs
            .is_some_and(|next_timer| now >= next_timer);

    let mut health_failure_recorded = false;
    let mut cooldown_active = false;
    if let Ok(snapshot) = RuntimeService::new(context).status().await
        && let Some(session) = snapshot.session
        && snapshot.pid_running
        && snapshot.inbound_health.has_unreachable_endpoint()
    {
        if !should_record_health_failure(&session) {
            cooldown_active = true;
            return HealthTickOutcome {
                health_failure_recorded: false,
                timer_due,
                cooldown_active,
            };
        }
        let failed_at = now_epoch_seconds();
        let cooldown_until = (failed_at + state.cooldown_secs).to_string();
        let failed_at = failed_at.to_string();
        let _ = context
            .db
            .update_runtime_session_transition_metadata(
                session.id,
                Some("daemon"),
                Some(&state.instance_id),
                Some("health_check_failed"),
                Some("runtime health check detected unreachable inbound endpoint"),
                Some("daemon"),
            )
            .await;
        let _ = context
            .db
            .update_runtime_session_failure_tracking(
                session.id,
                Some(&cooldown_until),
                Some(&failed_at),
                Some("health_check_failed"),
            )
            .await;
        health_failure_recorded = true;
    }
    HealthTickOutcome {
        health_failure_recorded,
        timer_due,
        cooldown_active,
    }
}

pub(super) fn should_record_health_failure(session: &crate::db::RuntimeSessionRecord) -> bool {
    if session.last_failed_reason_code.as_deref() != Some("health_check_failed") {
        return true;
    }
    let Some(cooldown_until) = session.cooldown_until.as_deref() else {
        return true;
    };
    let Ok(cooldown_until) = cooldown_until.parse::<u64>() else {
        return true;
    };
    now_epoch_seconds() >= cooldown_until
}
