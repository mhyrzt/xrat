use super::support::*;
use super::*;

#[tokio::test]
async fn replace_rejects_without_running_session() {
    let context = test_context().await;
    let result = RuntimeService::new(&context)
        .replace(ReplaceRequest {
            trigger: RotationTrigger::Manual,
            candidate_id: None,
        })
        .await;
    match result {
        Err(AppError::InvalidArgument(message)) => {
            assert!(message.contains("no running runtime session to replace"));
        }
        other => panic!("expected invalid argument, got {other:?}"),
    }
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
