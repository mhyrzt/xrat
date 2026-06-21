use super::super::super::import_cases::test_node;
use super::*;

#[tokio::test]
async fn list_with_latest_tests_returns_config_and_test() {
    let db_path = test_database_path("xrat-server-ops-joined");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("node-a")])
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
        icmp_ok: Some(true),
        icmp_ms: Some(40),
        tcp_ok: Some(true),
        tcp_ms: Some(50),
        real_delay_ok: Some(true),
        real_delay_ms: Some(200),
        download_mbps: None,
        upload_mbps: None,
        connect_ms: None,
        ttfb_ms: None,
        http_status: None,
        dial_endpoint_ip: None,
        dial_endpoint_location: Some("NL/North Holland/Amsterdam".to_string()),
        dial_endpoint_country: Some("NL".to_string()),
        dial_endpoint_asn: Some("AS60781 LeaseWeb".to_string()),
        dial_endpoint_geoip_source: None,
        dial_endpoint_fronting: None,
        failure_kind: None,
        failure_reason: None,
    })
    .await
    .expect("test should insert");

    let rows = db
        .list_configs_with_latest_tests(&ConfigListFilter::default())
        .await
        .expect("joined list should succeed");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].config.id, config.id);
    assert_eq!(rows[0].icmp_ok, Some(true));
    assert_eq!(rows[0].icmp_ms, Some(40));
    assert_eq!(rows[0].tcp_ok, Some(true));
    assert_eq!(rows[0].real_delay_ms, Some(200));
    assert_eq!(rows[0].dial_endpoint_country.as_deref(), Some("NL"));
    assert_eq!(
        rows[0].dial_endpoint_location.as_deref(),
        Some("NL/North Holland/Amsterdam")
    );
    assert_eq!(
        rows[0].dial_endpoint_asn.as_deref(),
        Some("AS60781 LeaseWeb")
    );
    assert!(rows[0].tested_at.is_some());

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn list_with_latest_tests_returns_null_test_when_no_test_exists() {
    let db_path = test_database_path("xrat-server-ops-no-test");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("node-b")])
        .await
        .expect("import should succeed");

    let rows = db
        .list_configs_with_latest_tests(&ConfigListFilter::default())
        .await
        .expect("joined list should succeed");

    assert_eq!(rows.len(), 1);
    assert!(rows[0].test_id.is_none());
    assert!(rows[0].real_delay_ms.is_none());
    assert!(rows[0].tested_at.is_none());

    let _ = std::fs::remove_file(db_path);
}
