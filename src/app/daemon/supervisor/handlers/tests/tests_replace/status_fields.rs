use super::super::super::test_support::test_context;
use super::*;
use crate::app::daemon::supervisor::{ProxyStatusResult, SupervisorEvent, SupervisorState};
use tokio::sync::oneshot;

#[tokio::test]
async fn proxy_status_reports_candidate_and_cooldown_fields() {
    let context = test_context("proxy-status-fields").await;
    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.last_trigger = Some(crate::app::daemon::ipc::RotationTrigger::Timer);
    state.last_result = "rotation_no_candidate".to_string();
    state.last_candidate_config_id = Some(42);
    state.last_candidate_result = "rotation_no_candidate".to_string();
    state.cooldown_active = true;

    let (tx, rx) = oneshot::channel();
    handle_event(
        &mut state,
        SupervisorEvent::ProxyStatus { respond_to: tx },
        &context,
    )
    .await;
    let payload = match rx.await.expect("proxy status should arrive") {
        ProxyStatusResult::Ok(payload) => payload,
        other => panic!("expected proxy status payload, got {other:?}"),
    };
    assert_eq!(payload.last_candidate_config_id, Some(42));
    assert_eq!(payload.last_candidate_result, "rotation_no_candidate");
    assert!(payload.cooldown_active);
}
