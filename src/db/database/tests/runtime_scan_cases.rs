use super::import_cases::test_node;
use super::postgres_cases::verify_database_backend;
use super::*;

#[tokio::test]
async fn stores_and_updates_runtime_sessions() {
    let db_path = test_database_path("xrat-runtime-sessions");
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

    let session_id = db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(config.id),
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

    let running = db
        .get_running_runtime_session()
        .await
        .expect("running session query should succeed")
        .expect("running session should exist");
    assert_eq!(running.id, session_id);
    assert_eq!(running.status, RuntimeSessionStatus::Starting);
    assert_eq!(running.socks_host.as_deref(), Some("127.0.0.1"));
    assert_eq!(running.socks_port, Some(10808));
    assert_eq!(running.http_host.as_deref(), Some("127.0.0.1"));
    assert_eq!(running.http_port, Some(18080));
    assert_eq!(running.shadowsocks_host.as_deref(), Some("127.0.0.1"));
    assert_eq!(running.shadowsocks_port, Some(1081));

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

    let latest = db
        .get_latest_runtime_session()
        .await
        .expect("latest session query should succeed")
        .expect("latest session should exist");

    assert_eq!(db.get_runtime_session_count().await.expect("count"), 1);
    assert_eq!(latest.id, session_id);
    assert_eq!(latest.status, RuntimeSessionStatus::Stopped);
    assert_eq!(latest.process_id, Some(4242));
    assert_eq!(latest.stopped_at.as_deref(), Some("2025-01-01T10:05:00Z"));
    assert!(
        db.get_running_runtime_session()
            .await
            .expect("running query should succeed")
            .is_none()
    );

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn upserts_and_lists_cf_scan_results() {
    let db_path = test_database_path("xrat-cf-scan");
    let db = Database::connect_sqlite(&db_path)
        .await
        .expect("db should open");

    db.upsert_cf_scan_results(&[
        CfScanResultUpsert {
            ip: "1.1.1.1".to_string(),
            latency_ms: Some(18),
            download_mbps: Some(93.0),
            upload_mbps: Some(14.2),
            error: None,
        },
        CfScanResultUpsert {
            ip: "1.0.0.1".to_string(),
            latency_ms: None,
            download_mbps: None,
            upload_mbps: None,
            error: Some("timeout".to_string()),
        },
    ])
    .await
    .expect("initial upsert should succeed");

    db.upsert_cf_scan_results(&[CfScanResultUpsert {
        ip: "1.1.1.1".to_string(),
        latency_ms: Some(12),
        download_mbps: None,
        upload_mbps: Some(15.0),
        error: None,
    }])
    .await
    .expect("update upsert should succeed");

    let all = db
        .list_cf_scan_results()
        .await
        .expect("list should succeed");
    assert_eq!(all.len(), 2);

    let best = all
        .iter()
        .find(|row| row.ip == "1.1.1.1")
        .expect("best row should exist");
    assert_eq!(best.latency_ms, Some(12));
    assert_eq!(best.download_mbps, Some(93.0));
    assert_eq!(best.upload_mbps, Some(15.0));
    assert_eq!(best.error, None);

    let history = db
        .list_cf_scan_history(10)
        .await
        .expect("history should load");
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].ip, "1.1.1.1");
    assert_eq!(history[1].ip, "1.0.0.1");

    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn verifies_postgres_backend_when_url_is_set() {
    let Ok(url) = std::env::var("XRAT_POSTGRES_TEST_URL") else {
        eprintln!("skipping PostgreSQL verification; set XRAT_POSTGRES_TEST_URL to run it");
        return;
    };
    let db = Database::connect_postgres_url(url)
        .await
        .expect("postgres db should open");

    db.clear_for_test().await.expect("postgres should clear");
    verify_database_backend(&db).await;
    db.clear_for_test().await.expect("postgres should clear");
}
