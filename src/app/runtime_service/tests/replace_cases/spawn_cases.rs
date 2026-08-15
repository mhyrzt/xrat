use super::fake_runtime::write_fake_runtime_script;
use super::support::*;
use super::*;

#[tokio::test]
async fn replace_spawn_failure_restores_old_runtime() {
    let mut context = test_context().await;
    let (active_config, failing_config) = import_two_configs(&context, "old", "fail").await;

    let mut child = spawn_sleep(5);
    let pid = i64::from(child.id());

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");
    let old_session_id = set_running_session(&context, active_config.id, pid).await;

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: Some(failing_config.id),
        })
        .await;

    assert!(
        result.is_err(),
        "replace should fail when candidate spawn fails"
    );
    let running = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running should load");
    let running = running.expect("the previous runtime should be restored");
    assert_eq!(running.config_id, Some(active_config.id));

    let restored_active = context
        .db
        .get_active_config()
        .await
        .expect("active config should load");
    assert_eq!(
        restored_active
            .expect("active config should be restored")
            .id,
        active_config.id
    );

    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_ne!(latest.id, old_session_id);

    if let Some(pid) = running.process_id {
        let _ = xray_runtime::terminate_process_gracefully(pid, SHUTDOWN_TIMEOUT);
    }
    assert_eq!(latest.status, RuntimeSessionStatus::Running);
    assert_eq!(
        context
            .db
            .get_runtime_session_count()
            .await
            .expect("session count should load"),
        3
    );
    assert_ne!(latest.id, old_session_id);

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn replace_health_trigger_reports_no_candidate_without_passing_test_result() {
    let context = test_context().await;
    let (active_config, alternate_config) = import_two_configs(&context, "c", "d").await;

    let mut child = spawn_sleep(5);
    let pid = i64::from(child.id());
    set_running_session(&context, active_config.id, pid).await;

    insert_failed_cooldown_session(&context, alternate_config.id, "1").await;

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::HealthCheckFailed,
            candidate_id: None,
        })
        .await;

    let _ = child.kill();
    let _ = child.wait();

    match result {
        Err(AppError::InvalidArgument(message)) => {
            assert!(
                message.contains("no eligible replacement candidate"),
                "expected no candidate without passing tests, got: {message}"
            );
        }
        other => panic!("expected invalid argument, got {other:?}"),
    }
}
