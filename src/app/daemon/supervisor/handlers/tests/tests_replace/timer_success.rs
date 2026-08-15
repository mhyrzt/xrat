use super::super::super::test_support::{test_context, test_node, test_source};
use super::helpers::{set_running_session, spawn_sleep, write_fake_runtime_script};
use super::*;
use crate::app::daemon::supervisor::{SupervisorEvent, SupervisorState};

#[tokio::test]
async fn health_tick_timer_due_success_updates_rotation_state_and_reschedules() {
    let mut context = test_context("health-tick-timer-success").await;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("TCP candidate listener should bind");
    let candidate_port = listener.local_addr().unwrap().port();
    let mut candidate_node = test_node("127.0.0.1", "b");
    candidate_node.port = candidate_port;
    context
        .db
        .import_nodes(
            &test_source(),
            &[test_node("example-a.com", "a"), candidate_node],
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

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");
    context.app_config.runtime.rotation.test_stages = vec!["tcp".to_string()];

    let mut old = spawn_sleep(30);
    let old_pid = i64::from(old.id());
    let _old_session_id = set_running_session(&context, active_config.id, old_pid).await;

    let mut state = SupervisorState::new("daemon-test".to_string());
    state.rotation_enabled = true;
    state.health_trigger_enabled = false;
    state.rotation_interval_secs = 600;
    state.next_timer_epoch_secs = Some(1);

    handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;

    assert_eq!(
        state.last_trigger,
        Some(crate::app::daemon::ipc::RotationTrigger::Timer)
    );
    assert_eq!(state.last_result, "replace_commit_success");
    assert_eq!(state.last_candidate_result, "replace_commit_success");
    assert!(!state.cooldown_active);
    assert!(state.next_timer_epoch_secs.is_some());

    let running = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running session should load")
        .expect("running session should exist");
    assert_eq!(state.last_candidate_config_id, running.config_id);
    assert_eq!(
        running.last_transition_reason_code.as_deref(),
        Some("replace_commit_success")
    );
    assert_ne!(running.process_id, Some(old_pid));

    let _ = crate::xray::process_mgmt::terminate_process_gracefully(
        running.process_id.unwrap_or_default(),
        std::time::Duration::from_millis(1500),
    );
    let _ = old.kill();
    let _ = old.wait();
    drop(listener);
}
