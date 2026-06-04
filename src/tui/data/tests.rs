use crate::db::{ConnectionTestRecord, ConnectionTestRunRecord, SubscriptionRecord};

use super::{TuiConfigRow, TuiData, TuiRuntimeStatus, TuiSourceRow, TuiTestStatus};

fn row(id: i64, delay: Option<i64>) -> TuiConfigRow {
    TuiConfigRow {
        id,
        name: format!("config-{id}"),
        protocol: "vless".to_string(),
        address: "example.com".to_string(),
        port: 443,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        real_delay_ms: delay,
        tcp_ms: Some(20),
        failure_reason: None,
        source_id: None,
        tested_at: Some("2026-01-01T00:00:00Z".to_string()),
        imported_at: "2026-01-01T00:00:00Z".to_string(),
        is_active: false,
        is_enabled: true,
        is_deleted: false,
    }
}

#[test]
fn summarizes_config_counts() {
    let mut failed = row(3, None);
    failed.failure_reason = Some("timeout".to_string());
    failed.is_enabled = false;

    let data = TuiData::from_configs(vec![row(1, Some(100)), row(2, Some(200)), failed]);

    assert_eq!(data.total_configs, 3);
    assert_eq!(data.enabled_configs, 2);
    assert_eq!(data.deleted_configs, 0);
    assert_eq!(data.failed_configs, 1);
}

#[test]
fn formats_network_and_delay_labels() {
    let mut active = row(4, Some(88));
    active.is_active = true;

    assert_eq!(active.network_label(), "ws+tls");
    assert_eq!(active.delay_label(), "88ms");
}

#[test]
fn matches_searchable_config_fields() {
    let row = row(4, Some(88));

    assert!(row.matches_search("config-4"));
    assert!(row.matches_search("vless"));
    assert!(row.matches_search("example"));
    assert!(!row.matches_search("missing"));
}

#[test]
fn maps_subscription_record_to_source_row() {
    let row = TuiSourceRow::from(SubscriptionRecord {
        id: 7,
        source_kind: "url".to_string(),
        source_url: Some("https://example.com/sub".to_string()),
        name: Some("main".to_string()),
        created_at: "created".to_string(),
        updated_at: "updated".to_string(),
        config_count: 42,
    });

    assert_eq!(row.id, 7);
    assert_eq!(row.display_name(), "main");
    assert_eq!(row.value_label(), "https://example.com/sub");
    assert_eq!(row.config_count, 42);
}

#[test]
fn default_runtime_status_is_renderable() {
    let runtime = TuiRuntimeStatus::default();

    assert_eq!(runtime.status, "unknown");
    assert_eq!(runtime.database_label, "-");
    assert!(!runtime.pid_running);
}

#[test]
fn summarizes_latest_test_run() {
    let mut untested = row(2, None);
    untested.tcp_ms = None;
    let configs = vec![row(1, Some(100)), untested];
    let failed = test_record(11, 1, Some("timeout"));
    let ok = test_record(12, 2, None);

    let status = TuiTestStatus::from_run_and_results(
        Some(ConnectionTestRunRecord {
            id: 5,
            kind: "real-delay".to_string(),
            created_at: "created".to_string(),
        }),
        vec![failed, ok],
        &configs,
    );

    assert_eq!(status.latest_run_id, Some(5));
    assert_eq!(status.total_results, 2);
    assert_eq!(status.success_results, 1);
    assert_eq!(status.failed_results, 1);
    assert_eq!(status.untested_configs, 1);
    assert_eq!(status.progress_label(), "2 done · 1 ok · 1 failed");
}

fn test_record(id: i64, config_id: i64, failure_reason: Option<&str>) -> ConnectionTestRecord {
    ConnectionTestRecord {
        id,
        run_id: Some(5),
        config_id,
        icmp_ok: None,
        icmp_ms: None,
        tcp_ok: Some(failure_reason.is_none()),
        tcp_ms: Some(20),
        real_delay_ok: Some(failure_reason.is_none()),
        real_delay_ms: Some(100),
        download_mbps: None,
        upload_mbps: None,
        connect_ms: None,
        ttfb_ms: None,
        http_status: None,
        endpoint_ip: None,
        endpoint_location: None,
        endpoint_country: None,
        endpoint_asn: None,
        failure_kind: None,
        failure_reason: failure_reason.map(str::to_string),
        tested_at: "tested".to_string(),
    }
}
