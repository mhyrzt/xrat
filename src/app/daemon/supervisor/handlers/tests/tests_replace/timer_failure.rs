use super::super::super::test_support::test_context;
use super::*;
use crate::app::daemon::supervisor::{SupervisorEvent, SupervisorState};

#[tokio::test]
async fn health_tick_timer_does_not_start_an_idle_runtime() {
    let context = test_context("health-tick-timer-due").await;
    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.next_timer_epoch_secs = Some(1);

    handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

    assert_eq!(state.last_trigger, None);
    assert_eq!(state.last_result, "never_triggered");
    assert_eq!(state.last_candidate_result, "never_selected");
}
