use super::super::super::test_support::test_context;
use super::*;
use crate::app::daemon::supervisor::{SupervisorEvent, SupervisorState};

#[tokio::test]
async fn health_tick_timer_due_attempt_updates_rotation_state_on_failure() {
    let context = test_context("health-tick-timer-due").await;
    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.next_timer_epoch_secs = Some(1);

    handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::ipc::RotationTrigger::Timer)
    );
    assert_eq!(state.last_result, "rotation_no_candidate");
    assert_eq!(state.last_candidate_result, "rotation_no_candidate");
}
