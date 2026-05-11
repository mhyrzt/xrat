use super::support::*;
use super::*;

#[tokio::test]
async fn stale_reconcile_preserves_cli_transition_origin_for_cli_owned_session() {
    let context = test_context().await;
    let config = import_single_config(&context).await;
    let session_id =
        insert_dead_pid_session(&context, config.id, RuntimeSessionStatus::Running).await;

    context
        .db
        .update_runtime_session_transition_metadata(
            session_id,
            Some("cli"),
            None,
            Some("manual_connect"),
            Some("runtime connect request succeeded"),
            Some("cli"),
        )
        .await
        .expect("transition metadata should update");

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
    assert_eq!(latest.last_transition_origin.as_deref(), Some("cli"));
    assert_eq!(latest.owner_kind.as_deref(), Some("cli"));
}
