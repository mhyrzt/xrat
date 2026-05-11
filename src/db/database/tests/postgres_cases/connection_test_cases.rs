use super::*;

pub(super) async fn verify_connection_test_state(db: &Database, config_id: i64) {
    db.insert_connection_test(&ConnectionTestInsert {
        run_id: None,
        config_id,
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
        .get_latest_connection_test(config_id)
        .await
        .expect("latest test should load")
        .expect("latest test should exist");
    assert_eq!(db.get_connection_test_count().await.expect("count"), 1);
    assert_eq!(latest_test.download_mbps, Some(42.5));
    assert_eq!(latest_test.upload_mbps, Some(11.25));
}
