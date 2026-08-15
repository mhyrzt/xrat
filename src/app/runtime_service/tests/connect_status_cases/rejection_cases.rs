use std::process::{Command, Stdio};

use super::support::import_single_config;
use super::*;

#[tokio::test]
async fn connect_rejects_soft_deleted_config() {
    let context = test_context().await;
    let config = import_single_config(&context).await;

    context
        .db
        .delete_config(config.id)
        .await
        .expect("delete should succeed");

    let result = RuntimeService::new(&context)
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await;

    match result {
        Err(AppError::InvalidArgument(message)) => {
            assert!(message.contains("deleted"));
        }
        other => panic!("expected invalid argument for deleted config, got {other:?}"),
    }
}

#[tokio::test]
async fn connect_rejects_when_runtime_running_and_replace_disabled() {
    let mut context = test_context().await;
    context.app_config.runtime.replace_active_session = false;
    let config = import_single_config(&context).await;

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

    let result = RuntimeService::new(&context)
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await;
    let _ = child.kill();
    let _ = child.wait();

    assert!(matches!(result, Err(AppError::RuntimeSessionAlreadyActive)));
}

#[tokio::test]
async fn connect_preflight_failure_keeps_running_session() {
    let mut context = test_context().await;
    context.app_config.runtime.replace_active_session = true;
    context.runtime_paths.xray_path = "/bin/false".into();
    let config = import_single_config(&context).await;

    let mut child = Command::new("sleep")
        .arg("5")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep process should spawn");
    let pid = i64::from(child.id());
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

    let result = RuntimeService::new(&context)
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await;

    assert!(
        matches!(result, Err(AppError::InvalidArgument(message)) if message.contains("native runtime config validation failed"))
    );
    assert!(
        child
            .try_wait()
            .expect("process state should load")
            .is_none()
    );
    let latest = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest session should load")
        .expect("session should exist");
    assert_eq!(latest.id, session_id);
    assert_eq!(latest.status, RuntimeSessionStatus::Running);

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn connect_rejects_non_hy2_sing_box_runtime_until_generation_exists() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    let config = import_single_config(&context).await;

    let result = RuntimeService::new(&context)
        .connect(ConnectRequest {
            config_id: config.id,
        })
        .await;

    match result {
        Err(AppError::InvalidArgument(message)) => {
            assert!(message.contains("supports hy2 configs only"));
        }
        other => panic!("expected invalid argument, got {other:?}"),
    }
}
