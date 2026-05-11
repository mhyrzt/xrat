use super::fake_runtime::write_fake_runtime_script;
use super::support::*;
use super::*;

#[tokio::test]
async fn replace_success_stages_new_then_stops_old_runtime() {
    let mut context = test_context().await;
    let config = import_single_config(&context).await;

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");

    let mut old = spawn_sleep(30);
    let old_pid = i64::from(old.id());

    let old_session_id = set_running_session(&context, config.id, old_pid).await;

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: Some(config.id),
        })
        .await
        .expect("replace should succeed");

    assert_eq!(result.old_session_id, old_session_id);
    assert_ne!(result.new_session_id, old_session_id);

    let mut old_stopped = false;
    for _ in 0..20 {
        if old
            .try_wait()
            .expect("old process wait should succeed")
            .is_some()
        {
            old_stopped = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(old_stopped, "old runtime process should be stopped");

    let running = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running should load")
        .expect("new running session should exist");
    assert_eq!(running.id, result.new_session_id);
    assert_eq!(running.status, RuntimeSessionStatus::Running);
    assert_eq!(running.config_id, Some(config.id));
    assert_eq!(
        running.last_transition_reason_code.as_deref(),
        Some("replace_commit_success")
    );
    assert_eq!(
        running.last_transition_reason_detail.as_deref(),
        Some("runtime replace handoff completed")
    );
    assert_eq!(running.last_transition_origin.as_deref(), Some("daemon"));

    let active_config = context
        .db
        .get_active_config()
        .await
        .expect("active config should load")
        .expect("active config should exist");
    assert_eq!(active_config.id, config.id);

    let _ = xray_runtime::terminate_process_gracefully(result.new_pid as i64, SHUTDOWN_TIMEOUT);
    let _ = old.kill();
    let _ = old.wait();
}
