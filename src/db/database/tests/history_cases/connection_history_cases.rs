use super::super::import_cases::test_node;
use super::super::*;

#[tokio::test]
async fn stores_and_reads_connection_test_history() {
    let db_path = test_database_path("xrat-connection-tests");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");
    let source = ImportSource {
        kind: SourceKind::File,
        value: "sample.txt".to_string(),
        name: None,
    };

    db.import_nodes(&source, &[test_node("first")])
        .await
        .expect("import should succeed");

    let config = db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("list should succeed")
        .into_iter()
        .next()
        .expect("config should exist");
    let run_id = db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "bulk".to_string(),
        })
        .await
        .expect("run insert should succeed");

    db.insert_connection_test(&ConnectionTestInsert {
        run_id: Some(run_id),
        config_id: config.id,
        icmp_ok: Some(false),
        icmp_ms: None,
        tcp_ok: Some(false),
        tcp_ms: None,
        real_delay_ok: None,
        real_delay_ms: None,
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
        failure_kind: Some("timeout".to_string()),
        failure_reason: Some("tcp handshake timed out".to_string()),
    })
    .await
    .expect("first test insert should succeed");

    db.insert_connection_test(&ConnectionTestInsert {
        run_id: Some(run_id),
        config_id: config.id,
        icmp_ok: Some(true),
        icmp_ms: Some(50),
        tcp_ok: Some(true),
        tcp_ms: Some(120),
        real_delay_ok: Some(true),
        real_delay_ms: Some(240),
        download_mbps: Some(42.5),
        upload_mbps: Some(11.25),
        connect_ms: Some(95),
        ttfb_ms: Some(180),
        http_status: Some(204),
        dial_endpoint_ip: Some("1.1.1.1".to_string()),
        dial_endpoint_location: Some("US".to_string()),
        dial_endpoint_country: Some("US".to_string()),
        dial_endpoint_asn: Some("AS13335 CLOUDFLARENET".to_string()),
        dial_endpoint_geoip_source: None,
        dial_endpoint_fronting: None,
        failure_kind: None,
        failure_reason: None,
    })
    .await
    .expect("second test insert should succeed");

    let tests = db
        .list_connection_tests(config.id)
        .await
        .expect("history should load");
    let run_tests = db
        .list_connection_tests_by_run(run_id)
        .await
        .expect("run history should load");
    let latest = db
        .get_latest_connection_test(config.id)
        .await
        .expect("latest should load")
        .expect("latest record should exist");

    assert_eq!(db.get_connection_test_count().await.expect("count"), 2);
    assert_eq!(tests.len(), 2);
    assert_eq!(run_tests.len(), 2);
    assert_eq!(latest.config_id, config.id);
    assert_eq!(latest.run_id, Some(run_id));
    assert_eq!(latest.tcp_ok, Some(true));
    assert_eq!(latest.tcp_ms, Some(120));
    assert_eq!(latest.real_delay_ok, Some(true));
    assert_eq!(latest.real_delay_ms, Some(240));
    assert_eq!(latest.download_mbps, Some(42.5));
    assert_eq!(latest.upload_mbps, Some(11.25));
    assert_eq!(latest.connect_ms, Some(95));
    assert_eq!(latest.ttfb_ms, Some(180));
    assert_eq!(latest.http_status, Some(204));
    assert_eq!(latest.dial_endpoint_ip.as_deref(), Some("1.1.1.1"));
    assert_eq!(latest.dial_endpoint_location.as_deref(), Some("US"));
    assert_eq!(latest.dial_endpoint_country.as_deref(), Some("US"));
    assert_eq!(
        latest.dial_endpoint_asn.as_deref(),
        Some("AS13335 CLOUDFLARENET")
    );
    assert_eq!(latest.failure_kind, None);
    assert_eq!(
        db.get_latest_connection_test_run()
            .await
            .expect("latest run should load")
            .expect("latest run should exist")
            .id,
        run_id
    );

    let _ = std::fs::remove_file(db_path);
}
