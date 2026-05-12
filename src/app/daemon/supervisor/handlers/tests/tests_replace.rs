use super::super::test_support::{test_context, test_node, test_source};
use super::super::*;
use crate::app::daemon::supervisor::{RuntimeReplaceResult, SupervisorEvent, SupervisorState};
use crate::db::RuntimeSessionInsert;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

#[tokio::test]
async fn health_tick_cooldown_blocks_health_replace_candidate_selection() {
    let context = test_context("health-tick-cooldown-block").await;
    context
        .db
        .import_nodes(
            &test_source(),
            &[
                test_node("example-a.com", "a"),
                test_node("example-b.com", "b"),
            ],
        )
        .await
        .expect("nodes should import");
    let mut configs = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load");
    configs.sort_by_key(|cfg| cfg.id);
    let active_config = configs[0].clone();
    let cooldown_candidate = configs[1].clone();

    context
        .db
        .set_active_config(cooldown_candidate.id)
        .await
        .expect("cooldown candidate should be active first");
    let cooldown_session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(cooldown_candidate.id),
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
        .expect("cooldown session should insert");

    let mut state = SupervisorState::new("daemon-test".to_string());
    handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

    let cooled = context
        .db
        .get_latest_runtime_session_for_config(cooldown_candidate.id)
        .await
        .expect("cooldown session should load")
        .expect("cooldown session should exist");
    assert_eq!(
        cooled.last_failed_reason_code.as_deref(),
        Some("health_check_failed")
    );
    assert!(cooled.cooldown_until.is_some());

    context
        .db
        .update_runtime_session_state(
            cooldown_session_id,
            crate::db::RuntimeSessionStatus::Failed,
            None,
            None,
            Some("2"),
            Some("simulate handoff away from cooled candidate"),
        )
        .await
        .expect("cooldown session should mark failed");

    context
        .db
        .set_active_config(active_config.id)
        .await
        .expect("active config should switch");
    context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(active_config.id),
            status: crate::db::RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(i64::from(std::process::id())),
            failure_reason: None,
            started_at: Some("3".to_string()),
            stopped_at: None,
        })
        .await
        .expect("active session should insert");

    let (tx, rx) = oneshot::channel();
    handle_event(
        &mut state,
        SupervisorEvent::RuntimeReplace {
            trigger: crate::app::daemon::server::RotationTrigger::HealthCheckFailed,
            candidate_id: None,
            respond_to: tx,
        },
        &context,
    )
    .await;

    match rx.await.expect("replace response should arrive") {
        RuntimeReplaceResult::Err { message } => {
            assert!(message.contains("no eligible replacement candidate"));
        }
        other => panic!("expected replace error, got {other:?}"),
    }
    assert_eq!(state.last_result, "rotation_no_candidate");
    assert_eq!(state.last_candidate_result, "rotation_no_candidate");
    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::server::RotationTrigger::HealthCheckFailed)
    );
}

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

#[tokio::test]
async fn health_tick_timer_due_attempt_updates_rotation_state_on_failure() {
    let context = test_context("health-tick-timer-due").await;
    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.next_timer_epoch_secs = Some(1);

    handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::server::RotationTrigger::Timer)
    );
    assert_eq!(state.last_result, "rotation_candidate_failed");
    assert_eq!(state.last_candidate_result, "rotation_candidate_failed");
}

#[tokio::test]
async fn manual_replace_failure_persists_rotation_reason_code_on_active_session() {
    let context = test_context("manual-replace-reason-code").await;
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
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be valid")
        .as_secs()
        .to_string();
    let session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: crate::db::RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(i64::from(std::process::id())),
            failure_reason: None,
            started_at: Some(now),
            stopped_at: None,
        })
        .await
        .expect("active session should insert");

    let mut state = SupervisorState::new("daemon-test".to_string());
    let (tx, rx) = oneshot::channel();
    handle_event(
        &mut state,
        SupervisorEvent::RuntimeReplace {
            trigger: crate::app::daemon::server::RotationTrigger::Manual,
            candidate_id: Some(-1),
            respond_to: tx,
        },
        &context,
    )
    .await;

    match rx.await.expect("replace response should arrive") {
        RuntimeReplaceResult::Err { message } => {
            assert!(message.contains("config -1 was not found"));
        }
        other => panic!("expected replace error, got {other:?}"),
    }

    let session = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("session should load")
        .expect("session should exist");
    assert_eq!(session.id, session_id);
    assert_eq!(
        session.last_transition_reason_code.as_deref(),
        Some("rotation_candidate_failed")
    );
    assert_eq!(state.last_result, "rotation_candidate_failed");
    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::server::RotationTrigger::Manual)
    );
}

#[tokio::test]
async fn proxy_status_reports_candidate_and_cooldown_fields() {
    let context = test_context("proxy-status-fields").await;
    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.last_trigger = Some(crate::app::daemon::server::RotationTrigger::Timer);
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
        crate::app::daemon::supervisor::ProxyStatusResult::Ok(payload) => payload,
        other => panic!("expected proxy status payload, got {other:?}"),
    };
    assert_eq!(payload.last_candidate_config_id, Some(42));
    assert_eq!(payload.last_candidate_result, "rotation_no_candidate");
    assert!(payload.cooldown_active);
}
