use super::super::super::import_cases::test_node;
use super::*;

#[tokio::test]
async fn protocol_filter_restricts_results() {
    let db_path = test_database_path("xrat-server-ops-protocol");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(
        &source,
        &[
            test_node("vless-node"),
            node_with_protocol("trojan", "trojan-node"),
        ],
    )
    .await
    .expect("import should succeed");

    let vless_filter = ConfigListFilter {
        protocol: Some("vless".to_string()),
        ..Default::default()
    };
    let vless_rows = db
        .list_configs_with_latest_tests(&vless_filter)
        .await
        .expect("vless filter should succeed");
    assert_eq!(vless_rows.len(), 1);
    assert_eq!(vless_rows[0].config.protocol, "vless");

    let trojan_filter = ConfigListFilter {
        protocol: Some("trojan".to_string()),
        ..Default::default()
    };
    let trojan_rows = db
        .list_configs_with_latest_tests(&trojan_filter)
        .await
        .expect("trojan filter should succeed");
    assert_eq!(trojan_rows.len(), 1);
    assert_eq!(trojan_rows[0].config.protocol, "trojan");

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn get_config_with_latest_test_returns_joined_record() {
    let db_path = test_database_path("xrat-server-ops-detail");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("detail-node")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");

    db.insert_connection_test(&ConnectionTestInsert {
        run_id: None,
        config_id: config.id,
        icmp_ok: None,
        icmp_ms: None,
        tcp_ok: Some(true),
        tcp_ms: Some(30),
        real_delay_ok: Some(true),
        real_delay_ms: Some(150),
        download_mbps: None,
        upload_mbps: None,
        connect_ms: None,
        ttfb_ms: None,
        http_status: None,
        dial_endpoint_ip: None,
        dial_endpoint_location: None,
        dial_endpoint_country: None,
        dial_endpoint_asn: None,
        dial_endpoint_geoip_source: None,
        dial_endpoint_fronting: None,
        failure_kind: None,
        failure_reason: None,
    })
    .await
    .expect("test should insert");

    let row = db
        .get_config_with_latest_test(config.id)
        .await
        .expect("detail should succeed")
        .expect("detail should exist");
    assert_eq!(row.config.id, config.id);
    assert_eq!(row.tcp_ok, Some(true));
    assert_eq!(row.real_delay_ms, Some(150));

    let missing = db
        .get_config_with_latest_test(-1)
        .await
        .expect("missing should succeed");
    assert!(missing.is_none());

    let _ = std::fs::remove_file(db_path);
}
