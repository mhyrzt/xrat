use super::super::*;

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
