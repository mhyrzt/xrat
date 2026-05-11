use super::super::test_support::{test_context, test_node, test_source};
use super::super::*;
use crate::app::daemon::supervisor::{RuntimeReplaceResult, SupervisorEvent, SupervisorState};
use crate::db::RuntimeSessionInsert;
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
}
