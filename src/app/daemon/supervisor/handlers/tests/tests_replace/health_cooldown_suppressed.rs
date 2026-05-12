use super::super::super::test_support::{test_context, test_node, test_source};
use super::*;
use crate::app::daemon::supervisor::{SupervisorEvent, SupervisorState};
use crate::db::RuntimeSessionInsert;

#[tokio::test]
async fn health_tick_sets_cooldown_active_when_failure_is_suppressed() {
    let context = test_context("health-tick-cooldown-suppressed").await;
    context
        .db
        .import_nodes(&test_source(), &[test_node("example-a.com", "a")])
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
        .set_active_config(config.id)
        .await
        .expect("active config should set");
    let session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: crate::db::RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(9),
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
        .expect("session should insert");
    context
        .db
        .update_runtime_session_failure_tracking(
            session_id,
            Some(&(u64::MAX - 1).to_string()),
            Some("1"),
            Some("health_check_failed"),
        )
        .await
        .expect("failure tracking should update");

    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.health_trigger_enabled = true;
    handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

    assert!(state.cooldown_active);
    assert!(state.last_trigger.is_none());
}
