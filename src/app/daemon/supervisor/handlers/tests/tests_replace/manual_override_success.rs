use super::super::super::test_support::{test_context, test_node, test_source};
use super::helpers::{set_running_session, spawn_sleep, write_fake_runtime_script};
use super::*;
use crate::app::daemon::supervisor::{RuntimeReplaceResult, SupervisorEvent, SupervisorState};
use crate::db::RuntimeSessionInsert;
use tokio::sync::oneshot;

#[tokio::test]
async fn manual_rotate_with_explicit_candidate_overrides_cooldown() {
    let mut context = test_context("manual-rotate-cooldown-override").await;
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
    let candidate_config = configs[1].clone();

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");

    let mut old = spawn_sleep(30);
    let old_pid = i64::from(old.id());
    let _old_session_id = set_running_session(&context, active_config.id, old_pid).await;

    let candidate_failed_session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(candidate_config.id),
            status: crate::db::RuntimeSessionStatus::Failed,
            socks_host: None,
            socks_port: None,
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: None,
            failure_reason: Some("health check failed".to_string()),
            started_at: None,
            stopped_at: Some("1".to_string()),
        })
        .await
        .expect("candidate failed session should insert");
    context
        .db
        .update_runtime_session_failure_tracking(
            candidate_failed_session_id,
            Some(&(u64::MAX - 1).to_string()),
            Some("1"),
            Some("health_check_failed"),
        )
        .await
        .expect("candidate cooldown should update");

    let mut state = SupervisorState::new("daemon-test".to_string());
    let (tx, rx) = oneshot::channel();
    handle_event(
        &mut state,
        SupervisorEvent::RuntimeReplace {
            trigger: crate::app::daemon::ipc::RotationTrigger::Manual,
            candidate_id: Some(candidate_config.id),
            respond_to: tx,
        },
        &context,
    )
    .await;
    let payload = match rx.await.expect("replace response should arrive") {
        RuntimeReplaceResult::Ok(payload) => payload,
        other => panic!("expected replace success, got {other:?}"),
    };

    assert_eq!(payload.new_config_id, candidate_config.id);
    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::ipc::RotationTrigger::Manual)
    );
    assert_eq!(state.last_result, "replace_commit_success");
    assert_eq!(state.last_candidate_config_id, Some(candidate_config.id));
    assert_eq!(state.last_candidate_result, "replace_commit_success");

    let _ = crate::xray::process_mgmt::terminate_process_gracefully(
        i64::from(payload.new_pid),
        std::time::Duration::from_millis(1500),
    );
    let _ = old.kill();
    let _ = old.wait();
}
