use super::fake_runtime::write_fake_runtime_script;
use super::support::*;
use super::*;

#[tokio::test]
async fn replace_starts_runtime_without_running_session() {
    let mut context = test_context().await;
    let config = import_single_config(&context).await;
    insert_passing_test(&context, config.id, 100).await;

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: Some(config.id),
        })
        .await
        .expect("replace should start a runtime when none is running");

    assert_eq!(result.old_session_id, None);
    assert_eq!(result.new_config_id, config.id);

    let running = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running session should load")
        .expect("running session should exist");
    assert_eq!(running.id, result.new_session_id);
    assert_eq!(running.config_id, Some(config.id));
    assert_eq!(
        running.last_transition_reason_code.as_deref(),
        Some("replace_commit_success")
    );
    assert_eq!(
        running.last_transition_reason_detail.as_deref(),
        Some("runtime rotation started new session")
    );
    assert_eq!(running.last_transition_origin.as_deref(), Some("daemon"));

    let _ = xray_runtime::terminate_process_gracefully(result.new_pid as i64, SHUTDOWN_TIMEOUT);
}

#[tokio::test]
async fn replace_rejects_without_running_session_or_candidate() {
    let context = test_context().await;
    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: None,
        })
        .await;
    match result {
        Err(AppError::InvalidArgument(message)) => {
            assert!(message.contains("no eligible replacement candidate"));
        }
        other => panic!("expected invalid argument, got {other:?}"),
    }
}

#[tokio::test]
async fn manual_rotate_excludes_config_without_passing_real_delay() {
    let context = test_context().await;
    let config = import_single_config(&context).await;
    insert_failing_test(&context, config.id).await;

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: None,
        })
        .await;

    match result {
        Err(AppError::InvalidArgument(message)) => {
            assert!(
                message.contains("no eligible replacement candidate"),
                "expected no candidate without a passing real-delay, got: {message}"
            );
        }
        other => panic!("expected invalid argument, got {other:?}"),
    }
}

#[tokio::test]
async fn manual_rotate_does_not_reuse_stored_tcp_result() {
    let mut context = test_context().await;
    let config = import_single_config(&context).await;
    insert_tcp_passing_test(&context, config.id, 50).await;

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: None,
        })
        .await;

    assert!(
        matches!(result, Err(AppError::InvalidArgument(message)) if message.contains("fresh configured rotation tests"))
    );
}

#[tokio::test]
async fn manual_rotate_does_not_reuse_stored_real_delay_result() {
    let mut context = test_context().await;
    let (untested, tested) = import_two_configs(&context, "a", "b").await;
    insert_passing_test(&context, tested.id, 100).await;

    write_fake_runtime_script(&context);
    context.runtime_paths.xray_path = context.runtime_paths.root_dir.join("fake-xray.py");

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: None,
        })
        .await;

    let _ = (untested, tested);
    assert!(
        matches!(result, Err(AppError::InvalidArgument(message)) if message.contains("fresh configured rotation tests"))
    );
}

#[tokio::test]
async fn replace_validation_failure_keeps_old_runtime_active() {
    let context = test_context().await;
    let config = import_single_config(&context).await;

    let mut child = spawn_sleep(5);
    let pid = i64::from(child.id());

    let session_id = set_running_session(&context, config.id, pid).await;

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: Some(-1),
        })
        .await;

    let _ = child.kill();
    let _ = child.wait();

    match result {
        Err(AppError::InvalidArgument(message)) => {
            assert!(message.contains("config -1 was not found"));
        }
        other => panic!("expected invalid argument, got {other:?}"),
    }

    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_eq!(latest.id, session_id);
    assert_eq!(latest.status, RuntimeSessionStatus::Running);
    assert_eq!(latest.process_id, Some(pid));
    assert_eq!(
        latest.last_transition_reason_code.as_deref(),
        Some("replace_rollback_keep_old")
    );
    assert_eq!(
        latest.last_transition_reason_detail.as_deref(),
        Some("replacement candidate rejected before handoff")
    );
    assert_eq!(latest.last_transition_origin.as_deref(), Some("daemon"));

    let active_config = context
        .db
        .get_active_config()
        .await
        .expect("active config should load")
        .expect("active config should still exist");
    assert_eq!(active_config.id, config.id);
}

#[tokio::test]
async fn replace_health_trigger_rejects_when_only_alternative_is_on_cooldown() {
    let context = test_context().await;
    let (active_config, alternate_config) = import_two_configs(&context, "a", "b").await;

    let mut child = spawn_sleep(5);
    let pid = i64::from(child.id());
    set_running_session(&context, active_config.id, pid).await;
    insert_failed_cooldown_session(&context, alternate_config.id, &(u64::MAX - 1).to_string())
        .await;

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
            assert!(message.contains("no eligible replacement candidate"));
        }
        other => panic!("expected invalid argument, got {other:?}"),
    }
}
