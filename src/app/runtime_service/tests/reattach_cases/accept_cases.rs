use super::*;

#[tokio::test]
async fn reattach_accepts_matching_pid_exec_cmdline() {
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
            process_id: Some(12345),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert");

    RuntimeService::new(&context)
        .reconcile_reattach_with_inspector(&AcceptingInspector, "daemon-test")
        .await
        .expect("reattach reconcile should succeed");

    let session = context
        .db
        .get_latest_runtime_session()
        .await
        .expect("latest should load")
        .expect("latest should exist");
    assert_eq!(session.id, session_id);
    assert_eq!(session.status, RuntimeSessionStatus::Running);
    assert!(session.failure_reason.is_none());
    assert_eq!(session.owner_kind.as_deref(), Some("daemon"));
    assert_eq!(session.owner_instance_id.as_deref(), Some("daemon-test"));
    assert_eq!(
        session.last_transition_reason_code.as_deref(),
        Some("daemon_restart_reattach_ok")
    );
    assert_eq!(
        context
            .db
            .get_active_config()
            .await
            .expect("active should load")
            .map(|row| row.id),
        Some(config.id)
    );
}
