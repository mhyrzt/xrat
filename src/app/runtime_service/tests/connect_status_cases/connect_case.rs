use super::*;
use std::process::{Command, Stdio};

#[tokio::test]
async fn connect_rejects_when_runtime_running_and_replace_disabled() {
    let mut context = test_context().await;
    context.app_config.runtime.replace_active_session = false;
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
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
    assert_eq!(summary.imported_configs, 1);

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
