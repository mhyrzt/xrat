use super::super::*;
use super::test_support::{test_context, test_node, test_source};

#[tokio::test]
async fn marks_running_session_with_dead_pid_as_failed() {
    let context = test_context().await;
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
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
    assert_eq!(summary.imported_configs, 1);
    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should be set");
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
            process_id: Some(0),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert");

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
    assert!(
        context
            .db
            .get_active_config()
            .await
            .expect("active should load")
            .is_none()
    );
}

#[tokio::test]
async fn marks_stopping_session_with_dead_pid_as_stopped() {
    let context = test_context().await;
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
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
    assert_eq!(summary.imported_configs, 1);
    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should be set");
    context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: RuntimeSessionStatus::Stopping,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(0),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert");

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
    assert!(
        context
            .db
            .get_active_config()
            .await
            .expect("active should load")
            .is_none()
    );
}
