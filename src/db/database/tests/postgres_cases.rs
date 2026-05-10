use super::import_cases::test_node;
use super::*;

pub(super) async fn verify_database_backend(db: &Database) {
    let source = ImportSource {
        kind: SourceKind::Url,
        value: "https://example.com/sub".to_string(),
        name: Some("Example".to_string()),
    };
    let first = test_node("first");
    let mut second = test_node("second");
    second.address = "second.example.com".to_string();
    second.uuid = Some("uuid-456".to_string());
    second.raw_config =
        "vless://uuid-456@second.example.com:443?type=ws&security=tls#second".to_string();

    let summary = db
        .import_nodes(&source, &[first, second])
        .await
        .expect("import should succeed");
    assert_eq!(summary.imported_configs, 2);
    assert_eq!(summary.total_configs, 2);
    assert_eq!(db.get_subscription_count().await.expect("count"), 1);
    assert_eq!(db.get_config_count().await.expect("count"), 2);

    let subscriptions = db.list_subscriptions().await.expect("subscriptions");
    assert_eq!(subscriptions.len(), 1);
    assert_eq!(subscriptions[0].source_kind, "url");
    assert_eq!(subscriptions[0].config_count, 2);

    let configs = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("configs should load");
    let first_id = configs[0].id;
    let second_id = configs[1].id;

    db.set_selected_config(first_id)
        .await
        .expect("select first should succeed");
    db.set_selected_config(second_id)
        .await
        .expect("select second should succeed");
    db.set_active_config(first_id)
        .await
        .expect("activate first should succeed");
    db.set_active_config(second_id)
        .await
        .expect("activate second should succeed");

    let selected = db
        .get_selected_config()
        .await
        .expect("selected query should succeed")
        .expect("selected config should exist");
    let active = db
        .get_active_config()
        .await
        .expect("active query should succeed")
        .expect("active config should exist");
    assert_eq!(selected.id, second_id);
    assert_eq!(active.id, second_id);

    db.set_config_enabled(second_id, false)
        .await
        .expect("disable should succeed");
    let enabled_configs = db
        .list_configs(&ConfigListFilter {
            only_enabled: true,
            ..ConfigListFilter::default()
        })
        .await
        .expect("enabled configs should load");
    assert_eq!(enabled_configs.len(), 1);
    assert!(db.get_selected_config().await.expect("selected").is_none());
    assert!(db.get_active_config().await.expect("active").is_none());

    db.set_config_enabled(second_id, true)
        .await
        .expect("enable should succeed");
    db.delete_config(first_id)
        .await
        .expect("delete should succeed");
    assert!(
        db.get_config_by_id(first_id)
            .await
            .expect("deleted query should succeed")
            .is_none()
    );

    db.insert_connection_test(&ConnectionTestInsert {
        run_id: None,
        config_id: second_id,
        icmp_ok: Some(true),
        icmp_ms: Some(50),
        tcp_ok: Some(true),
        tcp_ms: Some(120),
        real_delay_ok: Some(true),
        real_delay_ms: Some(240),
        download_mbps: Some(42.5),
        upload_mbps: Some(11.25),
        connect_ms: None,
        ttfb_ms: None,
        http_status: None,
        endpoint_ip: None,
        endpoint_location: None,
        endpoint_country: None,
        endpoint_asn: None,
        failure_kind: None,
        failure_reason: None,
    })
    .await
    .expect("connection test insert should succeed");
    let latest_test = db
        .get_latest_connection_test(second_id)
        .await
        .expect("latest test should load")
        .expect("latest test should exist");
    assert_eq!(db.get_connection_test_count().await.expect("count"), 1);
    assert_eq!(latest_test.download_mbps, Some(42.5));
    assert_eq!(latest_test.upload_mbps, Some(11.25));

    let session_id = db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(second_id),
            status: RuntimeSessionStatus::Starting,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(10808),
            http_host: Some("127.0.0.1".to_string()),
            http_port: Some(18080),
            shadowsocks_host: Some("127.0.0.1".to_string()),
            shadowsocks_port: Some(1081),
            process_id: None,
            failure_reason: None,
            started_at: Some("2025-01-01T10:00:00Z".to_string()),
            stopped_at: None,
        })
        .await
        .expect("runtime session insert should succeed");
    db.update_runtime_session_state(
        session_id,
        RuntimeSessionStatus::Running,
        Some(4242),
        None,
        None,
        None,
    )
    .await
    .expect("runtime session update should succeed");
    db.mark_runtime_session_stopped(session_id, Some("2025-01-01T10:05:00Z"))
        .await
        .expect("runtime session stop should succeed");

    let latest_session = db
        .get_latest_runtime_session()
        .await
        .expect("latest session should load")
        .expect("latest session should exist");
    assert_eq!(db.get_runtime_session_count().await.expect("count"), 1);
    assert_eq!(latest_session.status, RuntimeSessionStatus::Stopped);
    assert_eq!(latest_session.process_id, Some(4242));
    assert_eq!(latest_session.socks_port, Some(10808));
    assert_eq!(latest_session.http_port, Some(18080));
    assert_eq!(latest_session.shadowsocks_port, Some(1081));
    assert_eq!(
        latest_session.stopped_at.as_deref(),
        Some("2025-01-01T10:05:00Z")
    );
}
