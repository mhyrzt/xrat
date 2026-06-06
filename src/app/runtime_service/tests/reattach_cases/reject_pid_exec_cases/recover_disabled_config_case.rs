use super::super::*;

#[tokio::test]
async fn pid_missing_with_disabled_config_does_not_signal_recovery() {
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

    context
        .db
        .set_config_enabled(config.id, false)
        .await
        .expect("config should disable");
    context
        .db
        .set_active_config(config.id)
        .await
        .expect("active config should be set");
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
            process_id: None,
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert");

    let recovery = RuntimeService::new(&context)
        .reconcile_reattach_on_daemon_start("daemon-test")
        .await
        .expect("reattach reconcile should succeed");

    assert_eq!(
        recovery, None,
        "a disabled persisted config must not be auto-relaunched"
    );
}
