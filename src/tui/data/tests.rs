use crate::db::{ConnectionTestRecord, ConnectionTestRunRecord, SubscriptionRecord};

use super::{TuiConfigRow, TuiData, TuiRuntimeStatus, TuiSourceRow, TuiTestStatus};

fn row(id: i64, delay: Option<i64>) -> TuiConfigRow {
    TuiConfigRow {
        id,
        r#ref: format!("ref{id:09}"),
        name: format!("config-{id}"),
        protocol: "vless".to_string(),
        address: "example.com".to_string(),
        port: 443,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        icmp_ms: Some(10),
        real_delay_ms: delay,
        tcp_ms: Some(20),
        download_mbps: Some(42.25),
        upload_mbps: Some(11.5),
        endpoint_country: Some("NL".to_string()),
        endpoint_location: Some("NL/North Holland/Amsterdam".to_string()),
        endpoint_asn: Some("AS60781 LeaseWeb".to_string()),
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
    assert_eq!(active.icmp_label(), "10ms");
    assert_eq!(active.tcp_label(), "20ms");
    assert_eq!(active.download_label(), "42.2 Mbps");
    assert_eq!(active.upload_detail_label(), "11.50 Mbps");
    assert_eq!(active.country_label(), "NL");
}

#[test]
fn clears_test_fields_for_selected_configs() {
    let mut data = TuiData::from_configs(vec![row(1, Some(100)), row(2, Some(200))]);

    data.clear_test_fields_for_configs(&[2]);

    let cleared = data.configs.iter().find(|config| config.id == 2).unwrap();
    assert_eq!(cleared.delay_label(), "-");
    assert_eq!(cleared.icmp_label(), "-");
    assert_eq!(cleared.tcp_label(), "-");
    assert_eq!(cleared.download_label(), "-");
    assert_eq!(cleared.country_label(), "-");
    assert!(cleared.tested_at.is_none());

    let untouched = data.configs.iter().find(|config| config.id == 1).unwrap();
    assert_eq!(untouched.delay_label(), "100ms");
    assert_eq!(untouched.country_label(), "NL");
}

#[test]
fn metric_columns_follow_enabled_settings_when_available() {
    let mut settings = crate::app::config::TestingSettings::default();
    settings.icmp.enabled = false;
    settings.tcp.enabled = true;
    settings.real_delay.enabled = true;
    settings.download.enabled = false;
    settings.geoip.enabled = false;

    let columns = super::TuiMetricColumns::from_configs(&[row(1, Some(100))], Some(&settings));

    assert!(!columns.icmp);
    assert!(columns.tcp);
    assert!(columns.real_delay);
    assert!(!columns.download);
    assert!(columns.country);
}

#[test]
fn metric_columns_hide_location_when_geoip_disabled_and_no_location_data() {
    let mut settings = crate::app::config::TestingSettings::default();
    settings.geoip.enabled = false;
    let mut config = row(1, Some(100));
    config.endpoint_country = None;
    config.endpoint_location = None;
    config.endpoint_asn = None;

    let columns = super::TuiMetricColumns::from_configs(&[config], Some(&settings));

    assert!(!columns.country);
}

#[test]
fn matches_searchable_config_fields() {
    let row = row(4, Some(88));

    assert!(row.matches_search("config-4"));
    assert!(row.matches_search("ref000000004"));
    assert!(row.matches_search("vless"));
    assert!(row.matches_search("example"));
    assert!(row.matches_search("amsterdam"));
    assert!(row.matches_search("leaseweb"));
    assert!(!row.matches_search("missing"));
}

#[test]
fn applies_location_meta_overwriting_stale_values() {
    let mut row = row(5, Some(90));
    row.endpoint_country = Some("NL".to_string());
    row.endpoint_location = Some("loopback_ipv4".to_string());
    row.endpoint_asn = None;

    assert!(row.needs_location_enrichment());

    row.apply_location_meta(crate::support::geoip::EndpointGeoMeta {
        country: Some("US".to_string()),
        location: Some("US/California/Los Angeles".to_string()),
        asn: Some("AS15169 GOOGLE".to_string()),
    });

    assert_eq!(row.country_label(), "US");
    assert_eq!(row.location_label(), "US/California/Los Angeles");
    assert_eq!(row.asn_label(), "AS15169 GOOGLE");
}

#[test]
fn ignores_empty_location_meta() {
    let mut row = row(5, Some(90));
    row.endpoint_location = Some("loopback_ipv4".to_string());

    row.apply_location_meta(crate::support::geoip::EndpointGeoMeta::default());

    assert_eq!(row.location_label(), "loopback_ipv4");
}

#[test]
fn maps_subscription_record_to_source_row() {
    let row = TuiSourceRow::from(SubscriptionRecord {
        id: 7,
        r#ref: "ref000000007".to_string(),
        source_kind: "url".to_string(),
        source_url: Some("https://example.com/sub".to_string()),
        name: Some("main".to_string()),
        created_at: "created".to_string(),
        updated_at: "updated".to_string(),
        config_count: 42,
    });

    assert_eq!(row.id, 7);
    assert_eq!(row.display_ref(), "ref00000");
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

#[test]
fn omits_zero_counts_from_test_progress_label() {
    let configs = vec![row(1, Some(100)), row(2, Some(200))];
    let run = Some(ConnectionTestRunRecord {
        id: 5,
        kind: "real-delay".to_string(),
        created_at: "created".to_string(),
    });

    let all_ok = TuiTestStatus::from_run_and_results(
        run.clone(),
        vec![test_record(11, 1, None), test_record(12, 2, None)],
        &configs,
    );
    assert_eq!(all_ok.progress_label(), "2 done · 2 ok");

    let all_failed = TuiTestStatus::from_run_and_results(
        run,
        vec![
            test_record(11, 1, Some("timeout")),
            test_record(12, 2, Some("timeout")),
        ],
        &configs,
    );
    assert_eq!(all_failed.progress_label(), "2 done · 2 failed");
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
