use super::super::health::handle_probe_completed;
use super::super::test_support::{test_context, test_node, test_source};
use crate::app::daemon::supervisor::SupervisorState;
use crate::db::{RuntimeSessionInsert, RuntimeSessionStatus};

async fn insert_running_session(context: &crate::app::context::AppContext) -> i64 {
    context
        .db
        .import_nodes(&test_source(), &[test_node("example.com", "active")])
        .await
        .expect("node should import");
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");
    context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(i64::from(std::process::id())),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("runtime session should insert")
}

#[tokio::test]
async fn proxied_health_failure_requires_configured_consecutive_threshold() {
    let context = test_context("health-threshold").await;
    let session_id = insert_running_session(&context).await;
    let mut state = SupervisorState::new("daemon-test".to_string());
    state.health_failure_threshold = 3;

    for expected in [1, 2] {
        assert!(
            !handle_probe_completed(
                &mut state,
                &context,
                session_id,
                false,
                Some("request failed".to_string()),
            )
            .await
        );
        assert_eq!(state.consecutive_health_failures, expected);
    }
    assert!(
        handle_probe_completed(
            &mut state,
            &context,
            session_id,
            false,
            Some("request failed".to_string()),
        )
        .await
    );
    assert_eq!(state.consecutive_health_failures, 0);
    assert!(state.pending_health_recovery);
}

#[tokio::test]
async fn stale_health_probe_result_does_not_change_current_health_state() {
    let context = test_context("health-stale-result").await;
    let session_id = insert_running_session(&context).await;
    let mut state = SupervisorState::new("daemon-test".to_string());
    state.consecutive_health_failures = 1;
    state.last_health_error = Some("current session error".to_string());

    assert!(
        !handle_probe_completed(
            &mut state,
            &context,
            session_id + 999,
            false,
            Some("stale error".to_string()),
        )
        .await
    );
    assert_eq!(state.consecutive_health_failures, 1);
    assert_eq!(
        state.last_health_error.as_deref(),
        Some("current session error")
    );
    assert!(!state.pending_health_recovery);
}
