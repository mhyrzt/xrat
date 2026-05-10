use super::super::*;
use super::test_support::{test_context, test_node, test_source};
use crate::app::daemon::server::RotationTrigger;
use std::process::{Command, Stdio};

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
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
        .await
        .expect("node should import");
    assert_eq!(summary.imported_configs, 1);
    let config = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist");

    let mut child = Command::new("sleep")
        .arg("5")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep process should spawn");
    let pid = i64::from(child.id());

    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should be set");
    let session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
            status: RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(pid),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert");

    // Simulate validation failure by requesting a config that does not exist.
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

    let active_config = context
        .db
        .get_active_config()
        .await
        .expect("active config should load")
        .expect("active config should still exist");
    assert_eq!(active_config.id, config.id);
}
