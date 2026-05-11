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

pub(super) fn spawn_sleep(seconds: u64) -> Child {
    Command::new("sleep")
        .arg(seconds.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("sleep process should spawn")
}

pub(super) async fn set_running_session(context: &AppContext, config_id: i64, pid: i64) -> i64 {
    context
        .db
        .set_active_config(config_id)
        .await
        .expect("active config should be set");

    context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config_id),
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
        .expect("session should insert")
}

pub(super) async fn import_two_configs(
    context: &AppContext,
    left: &str,
    right: &str,
) -> (ConfigRecord, ConfigRecord) {
    let summary = context
        .db
        .import_nodes(
            &test_source(),
            &[
                test_node_with(&format!("example-{left}.com"), left),
                test_node_with(&format!("example-{right}.com"), right),
            ],
        )
        .await
        .expect("nodes should import");
    assert_eq!(summary.imported_configs, 2);

    let mut configs = context
        .db
        .list_configs(&Default::default())
        .await
        .expect("configs should load");
    configs.sort_by_key(|config| config.id);
    (configs[0].clone(), configs[1].clone())
}

pub(super) async fn insert_failed_cooldown_session(
    context: &AppContext,
    config_id: i64,
    first_failed_at: &str,
) -> i64 {
    let cooldown_session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config_id),
            status: RuntimeSessionStatus::Failed,
            socks_host: None,
            socks_port: None,
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: None,
            failure_reason: Some("health check failed".to_string()),
            started_at: None,
            stopped_at: Some("1".to_string()),
        })
        .await
        .expect("cooldown session should insert");

    context
        .db
        .update_runtime_session_failure_tracking(
            cooldown_session_id,
            Some(first_failed_at),
            Some("1"),
            Some("health_check_failed"),
        )
        .await
        .expect("cooldown tracking should update");

    cooldown_session_id
}
