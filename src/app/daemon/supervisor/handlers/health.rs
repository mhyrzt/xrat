use crate::app::daemon::supervisor::SupervisorState;
use crate::app::runtime::AppContext;
use crate::app::runtime_service::RuntimeService;
use std::time::{SystemTime, UNIX_EPOCH};

const HEALTH_FAILURE_COOLDOWN_SECONDS: u64 = 300;

pub(super) async fn handle_health_tick(state: &SupervisorState, context: &AppContext) {
    if let Ok(snapshot) = RuntimeService::new(context).status().await {
        if let Some(session) = snapshot.session {
            if snapshot.pid_running && snapshot.inbound_health.has_unreachable_endpoint() {
                if !should_record_health_failure(&session) {
                    return;
                }
                let failed_at = now_epoch_seconds();
                let cooldown_until = (failed_at + HEALTH_FAILURE_COOLDOWN_SECONDS).to_string();
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
            }
        }
    }
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
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
