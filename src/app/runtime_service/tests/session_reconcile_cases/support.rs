use super::*;

pub(super) async fn import_single_config(context: &AppContext) -> ConfigRecord {
    let summary = context
        .db
        .import_nodes(&test_source(), &[test_node()])
        .await
        .expect("node should import");
    assert_eq!(summary.imported_configs, 1);

    context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load")
        .into_iter()
        .next()
        .expect("config should exist")
}

pub(super) async fn insert_dead_pid_session(
    context: &AppContext,
    config_id: i64,
    status: RuntimeSessionStatus,
) -> i64 {
    context
        .db
        .set_active_config(config_id)
        .await
        .expect("active config should be set");

    context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config_id),
            status,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(0),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .expect("session should insert")
}

pub(super) async fn assert_active_config_cleared(context: &AppContext) {
    assert!(
        context
            .db
            .get_active_config()
            .await
            .expect("active should load")
            .is_none()
    );
}
