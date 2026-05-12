use super::super::super::test_support::{test_context, test_node, test_source};
use super::*;
use crate::app::daemon::supervisor::{RuntimeReplaceResult, SupervisorEvent, SupervisorState};
use crate::db::RuntimeSessionInsert;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::oneshot;

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
