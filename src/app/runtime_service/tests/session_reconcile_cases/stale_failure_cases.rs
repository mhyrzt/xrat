use super::support::*;
use super::*;

#[tokio::test]
async fn marks_running_session_with_dead_pid_as_failed() {
    let context = test_context().await;
    let config = import_single_config(&context).await;
    insert_dead_pid_session(&context, config.id, RuntimeSessionStatus::Running).await;

    let state = RuntimeService::new(&context)
        .active_session_state()
        .await
        .expect("session state should resolve");

    assert!(matches!(state, ActiveSessionState::Stale(_)));
    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_eq!(latest.status, RuntimeSessionStatus::Failed);
    assert_eq!(
        latest.last_transition_reason_code.as_deref(),
        Some("process_exit_unexpected")
    );
    assert_eq!(
        latest.last_failed_reason_code.as_deref(),
        Some("process_exit_unexpected")
    );
    assert!(latest.last_failed_at.is_some());
    assert_active_config_cleared(&context).await;
}

#[tokio::test]
async fn marks_stopping_session_with_dead_pid_as_stopped() {
    let context = test_context().await;
    let config = import_single_config(&context).await;
    insert_dead_pid_session(&context, config.id, RuntimeSessionStatus::Stopping).await;

    let state = RuntimeService::new(&context)
        .active_session_state()
        .await
        .expect("session state should resolve");

    assert!(matches!(state, ActiveSessionState::Stale(_)));
    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_eq!(latest.status, RuntimeSessionStatus::Stopped);
    assert_eq!(
        latest.last_transition_reason_code.as_deref(),
        Some("manual_disconnect")
    );
    assert!(latest.last_failed_reason_code.is_none());
    assert_active_config_cleared(&context).await;
}
