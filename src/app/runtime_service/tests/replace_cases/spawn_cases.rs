use super::support::*;
use super::*;

#[tokio::test]
async fn replace_spawn_failure_keeps_old_runtime_active() {
    let mut context = test_context().await;
    let config = import_single_config(&context).await;

    let mut child = spawn_sleep(5);
    let pid = i64::from(child.id());

    context.runtime_paths.xray_path = std::path::PathBuf::from("/definitely/missing-xray");
    let old_session_id = set_running_session(&context, config.id, pid).await;

    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: Some(config.id),
        })
        .await;

    let _ = child.kill();
    let _ = child.wait();

    assert!(
        result.is_err(),
        "replace should fail when candidate spawn fails"
    );
    let running = context
        .db
        .get_running_runtime_session()
        .await
        .expect("running should load")
        .expect("old runtime should still be running");
    assert_eq!(running.id, old_session_id);
    assert_eq!(running.status, RuntimeSessionStatus::Running);
    assert_eq!(running.process_id, Some(pid));

    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_ne!(latest.id, old_session_id);
    assert_eq!(latest.status, RuntimeSessionStatus::Failed);
    assert_eq!(
        latest.last_failed_reason_code.as_deref(),
        Some("replace_validation_failed")
    );
    assert!(latest.last_failed_at.is_some());
}

#[tokio::test]
async fn replace_health_trigger_uses_alternative_after_cooldown_expires() {
    let mut context = test_context().await;
    let (active_config, alternate_config) = import_two_configs(&context, "c", "d").await;

    let mut child = spawn_sleep(5);
    let pid = i64::from(child.id());
    set_running_session(&context, active_config.id, pid).await;

    insert_failed_cooldown_session(&context, alternate_config.id, "1").await;

    context.runtime_paths.xray_path = std::path::PathBuf::from("/definitely/missing-xray");
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
                !message.contains("no eligible replacement candidate"),
                "expected expired cooldown to allow candidate selection, got: {message}"
            );
        }
        Err(_) => {}
        Ok(_) => panic!("replace should not fully succeed with missing runtime binary"),
    }

    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_eq!(latest.config_id, Some(alternate_config.id));
    assert_eq!(latest.status, RuntimeSessionStatus::Failed);
    assert_eq!(
        latest.last_failed_reason_code.as_deref(),
        Some("replace_validation_failed")
    );
}
