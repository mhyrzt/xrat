use super::super::handlers::filter_latest_run_rows;
use super::super::*;

#[test]
fn formats_csv_results_with_download_speed() {
    let output = TestOutputRow {
        id: 7,
        r#ref: "abcdef123456".to_string(),
        name: Some("node, one".to_string()),
        protocol: "vless".to_string(),
        address: "example.com".to_string(),
        port: 443,
        icmp_ms: Some(12),
        real_delay_ms: Some(123),
        download_mbps: Some(45.678),
        upload_mbps: Some(12.345),
        status: TestStatus::Ok,
        error: None,
        tcp_ms: Some(25),
        ttfb_ms: Some(123),
        http_status: Some(204),
        endpoint_ip: Some("1.1.1.1".to_string()),
        endpoint_location: Some("US".to_string()),
        endpoint_country: Some("US".to_string()),
        endpoint_asn: Some("AS13335 CLOUDFLARENET".to_string()),
        ran_icmp: true,
        ran_tcp: true,
        ran_real_delay: true,
        icmp_ok: true,
        tcp_ok: true,
        real_delay_ok: true,
        failure_kind: None,
        elapsed_secs: 1.0,
    };

    let csv = format_csv(&[output]);

    assert!(csv.starts_with("ref,name,protocol,address,port,icmp_ms,real_delay_ms,download_mbps,upload_mbps,status,error\n"));
    assert!(
        csv.contains("abcdef123456,\"node, one\",vless,example.com,443,12,123,45.68,12.35,ok,")
    );
}

#[test]
fn resolves_configured_test_concurrency() {
    assert_eq!(resolve_concurrency(1).expect("positive concurrency"), 1);
    assert_eq!(resolve_concurrency(16).expect("positive concurrency"), 16);
    assert!(resolve_concurrency(-1).is_err());
    assert!(resolve_concurrency(0).expect("auto concurrency") >= 1);
}

#[test]
fn filters_latest_run_rows_by_country_and_asn() {
    let rows = vec![
        crate::db::ConnectionTestRecord {
            id: 1,
            run_id: Some(1),
            config_id: 10,
            icmp_ok: Some(true),
            icmp_ms: Some(10),
            tcp_ok: Some(true),
            tcp_ms: Some(20),
            real_delay_ok: Some(true),
            real_delay_ms: Some(30),
            download_mbps: Some(40.0),
            upload_mbps: Some(10.0),
            connect_ms: Some(20),
            ttfb_ms: Some(30),
            http_status: Some(200),
            endpoint_ip: Some("1.1.1.1".to_string()),
            endpoint_location: Some("US".to_string()),
            endpoint_country: Some("US".to_string()),
            endpoint_asn: Some("AS13335 CLOUDFLARENET".to_string()),
            failure_kind: None,
            failure_reason: None,
            tested_at: "now".to_string(),
        },
        crate::db::ConnectionTestRecord {
            id: 2,
            run_id: Some(1),
            config_id: 11,
            icmp_ok: Some(true),
            icmp_ms: Some(11),
            tcp_ok: Some(true),
            tcp_ms: Some(21),
            real_delay_ok: Some(true),
            real_delay_ms: Some(31),
            download_mbps: Some(41.0),
            upload_mbps: Some(11.0),
            connect_ms: Some(21),
            ttfb_ms: Some(31),
            http_status: Some(200),
            endpoint_ip: Some("8.8.8.8".to_string()),
            endpoint_location: Some("public".to_string()),
            endpoint_country: Some("DE".to_string()),
            endpoint_asn: Some("AS15169 GOOGLE".to_string()),
            failure_kind: None,
            failure_reason: None,
            tested_at: "now".to_string(),
        },
    ];

    let filtered = filter_latest_run_rows(rows, Some("us"), Some("cloudflare"));
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].config_id, 10);
}

#[test]
fn classifies_endpoint_location_from_ip() {
    assert_eq!(
        classify_endpoint_location(Some("10.0.0.7")).as_deref(),
        Some("private_ipv4")
    );
    assert_eq!(
        classify_endpoint_location(Some("8.8.8.8")).as_deref(),
        Some("public")
    );
    assert_eq!(
        classify_endpoint_location(Some("::1")).as_deref(),
        Some("loopback_ipv6")
    );
    assert!(classify_endpoint_location(Some("not-an-ip")).is_none());
    assert!(classify_endpoint_location(None).is_none());
}

#[tokio::test]
async fn resolves_endpoint_location_with_geoip_fallback() {
    let lookup = crate::support::geoip::LocalMmdbLookup::new(
        "/tmp/no-country.mmdb".into(),
        "/tmp/no-city.mmdb".into(),
        "/tmp/no-asn.mmdb".into(),
    );

    assert_eq!(
        resolve_endpoint_meta(Some("8.8.8.8"), true, &lookup,)
            .await
            .location
            .as_deref(),
        Some("public")
    );
    assert_eq!(
        resolve_endpoint_meta(Some("10.0.0.1"), false, &lookup,)
            .await
            .location
            .as_deref(),
        Some("private_ipv4")
    );
}
