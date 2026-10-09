use super::*;
use crate::app::read_models::ConfigDetail;
use crate::app::runtime_service::{
    RuntimeEndpoint, RuntimeEndpointHealth, RuntimeEndpointState, RuntimeInboundHealth,
    RuntimeSessionDisplay, RuntimeStatusSnapshot,
};
use crate::app::services::dashboard::{DaemonOverview, DashboardLogs, DashboardSnapshot};

#[test]
fn converts_dashboard_fixture_without_database_process_or_network_access() {
    let mut row = crate::app::services::test_support::sample_joined_row(7, false);
    row.config.is_deleted = true;
    row.test_id = Some(3);
    row.failure_reason = Some("failed".into());
    let data = TuiData::from(DashboardSnapshot {
        configs: vec![ConfigDetail::from_joined(&row)],
        sources: vec![],
        runtime: RuntimeStatusSnapshot {
            status: RuntimeSessionDisplay::Stopped,
            session: None,
            session_config: None,
            active_config: None,
            pid_running: false,
            database_label: "fake db".into(),
            inbound_health: RuntimeInboundHealth {
                http: Some(RuntimeEndpointHealth {
                    endpoint: RuntimeEndpoint {
                        host: "0.0.0.0".into(),
                        port: 8080,
                    },
                    state: RuntimeEndpointState::NotChecked,
                }),
                ..Default::default()
            },
        },
        tun_label: None,
        local_address: Some("192.0.2.10".into()),
        latest_run: None,
        test_results: vec![],
        logs: DashboardLogs::default(),
        probe_history: vec![],
        daemon: DaemonOverview {
            running: true,
            rotation_enabled: true,
            interval_secs: 60,
        },
        db_label: "fake db".into(),
        config_path: "/fake/config.toml".into(),
        api_b64_url: "http://192.0.2.10:18203/b64".into(),
        server_enabled: true,
        test_stages: vec!["icmp".into(), "real-delay".into(), "icmp".into()],
        pending_enrichment: vec![(7.into(), "example.com".into())],
    });
    assert_eq!(
        (
            data.total_configs,
            data.enabled_configs,
            data.deleted_configs,
            data.failed_configs
        ),
        (1, 0, 1, 1)
    );
    assert_eq!(
        data.runtime.http.as_deref(),
        Some("http://192.0.2.10:8080 ○")
    );
    assert_eq!(data.test_stage_label, "icmp + real-delay");
    assert!(data.metric_columns.icmp && data.metric_columns.real_delay);
    assert!(!data.metric_columns.tcp);
    assert_eq!(data.tests.untested_configs, 1);
    assert_eq!(data.tests.stale_configs, 1);
    assert!(data.daemon.running && data.daemon.rotation_enabled);
    assert_eq!(data.pending_enrichment.len(), 1);
}
