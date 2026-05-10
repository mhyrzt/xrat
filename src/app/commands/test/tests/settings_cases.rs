use super::super::*;
use super::geoip_cases::{test_args, test_runtime_paths};

use crate::app::config::{AppConfig, TestingSettings};
use crate::cli::TestArgs;

#[test]
fn rebuilds_node_from_config_record() {
    let record = ConfigRecord {
        id: 1,
        subscription_id: Some(2),
        dedup_key: "key".to_string(),
        protocol: "vmess".to_string(),
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("uuid-123".to_string()),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("cdn.example.com".to_string()),
        host: Some("cdn.example.com".to_string()),
        path: Some("/socket".to_string()),
        name: Some("node".to_string()),
        raw_config: "vmess://payload".to_string(),
        is_active: false,
        is_enabled: true,
        is_selected: false,
        imported_at: "now".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
    };

    let node = node_from_record(&record).expect("config record should rebuild");
    assert_eq!(node.protocol.as_str(), "vmess");
    assert_eq!(node.address, "example.com");
    assert_eq!(node.network, "ws");
    assert_eq!(node.uuid.as_deref(), Some("uuid-123"));
}

#[test]
fn resolves_test_settings_from_app_config() {
    let app_config = AppConfig {
        testing: TestingSettings {
            real_delay: crate::app::config::RealDelayTestSettings {
                enabled: true,
                url: "https://example.test/204".to_string(),
                timeout: 12_000,
            },
            download: crate::app::config::DownloadTestSettings {
                enabled: true,
                url: "https://example.test/10mb.test".to_string(),
                timeout: 40_000,
            },
            icmp: crate::app::config::IcmpTestSettings {
                enabled: true,
                attempts: 3,
                timeout: 2500,
            },
            tcp: crate::app::config::TcpTestSettings {
                enabled: true,
                timeout: 4500,
            },
            ..TestingSettings::default()
        },
        ..AppConfig::default()
    };
    let args = test_args(Some(1));

    let runtime_paths = test_runtime_paths();
    let settings = resolve_test_settings(&args, &app_config, &runtime_paths).expect("settings");

    assert_eq!(settings.real_delay_url, "https://example.test/204");
    assert_eq!(settings.download_url, "https://example.test/10mb.test");
    assert_eq!(settings.xray_binary_path, PathBuf::from("xray"));
    assert_eq!(settings.icmp_timeout, Duration::from_millis(2500));
    assert_eq!(settings.tcp_timeout, Duration::from_millis(4500));
    assert_eq!(settings.xray_startup_timeout, Duration::from_millis(5000));
    assert_eq!(settings.real_delay_timeout, Duration::from_millis(12_000));
    assert_eq!(settings.download_timeout, Duration::from_millis(40_000));
    assert_eq!(
        settings.stage_order,
        vec![
            ConnectionTestStage::Icmp,
            ConnectionTestStage::RealDelay,
            ConnectionTestStage::Download,
        ]
    );
    assert_eq!(settings.failure_policy, TestFailurePolicy::Continue);
}

#[test]
fn cli_test_settings_override_app_config() {
    let app_config = AppConfig {
        testing: TestingSettings {
            real_delay: crate::app::config::RealDelayTestSettings {
                enabled: true,
                url: "https://example.test/204".to_string(),
                timeout: 12_000,
            },
            icmp: crate::app::config::IcmpTestSettings {
                enabled: true,
                attempts: 3,
                timeout: 2500,
            },
            tcp: crate::app::config::TcpTestSettings {
                enabled: true,
                timeout: 4500,
            },
            ..TestingSettings::default()
        },
        ..AppConfig::default()
    };
    let args = TestArgs {
        test_url: Some("https://override.test/204".to_string()),
        download_url: Some("https://override.test/10mb.test".to_string()),
        icmp_timeout_ms: Some(3000),
        tcp_timeout_ms: Some(5000),
        real_delay_timeout_ms: Some(15_000),
        download_timeout_ms: Some(45_000),
        ..test_args(Some(1))
    };

    let runtime_paths = test_runtime_paths();
    let settings = resolve_test_settings(&args, &app_config, &runtime_paths).expect("settings");

    assert_eq!(settings.real_delay_url, "https://override.test/204");
    assert_eq!(settings.download_url, "https://override.test/10mb.test");
    assert_eq!(settings.icmp_timeout, Duration::from_millis(3000));
    assert_eq!(settings.tcp_timeout, Duration::from_millis(5000));
    assert_eq!(settings.real_delay_timeout, Duration::from_millis(15_000));
    assert_eq!(settings.download_timeout, Duration::from_millis(45_000));
}

#[test]
fn resolves_custom_test_stage_order() {
    let app_config = AppConfig {
        testing: TestingSettings {
            order: vec![ConnectionTestStage::RealDelay, ConnectionTestStage::Icmp],
            ..TestingSettings::default()
        },
        ..AppConfig::default()
    };
    let runtime_paths = test_runtime_paths();

    let settings =
        resolve_test_settings(&test_args(Some(1)), &app_config, &runtime_paths).expect("settings");

    assert_eq!(
        settings.stage_order,
        vec![ConnectionTestStage::RealDelay, ConnectionTestStage::Icmp]
    );
}

#[test]
fn rejects_duplicate_test_stage_order_entries() {
    let app_config = AppConfig {
        testing: TestingSettings {
            order: vec![ConnectionTestStage::Icmp, ConnectionTestStage::Icmp],
            ..TestingSettings::default()
        },
        ..AppConfig::default()
    };
    let runtime_paths = test_runtime_paths();

    let error = resolve_test_settings(&test_args(Some(1)), &app_config, &runtime_paths)
        .expect_err("duplicate stage should fail");

    assert!(error.to_string().contains("duplicate test stage"));
}

#[test]
fn resolves_configured_failure_policy() {
    let app_config = AppConfig {
        testing: TestingSettings {
            failure_policy: TestFailurePolicy::SkipRemaining,
            ..TestingSettings::default()
        },
        ..AppConfig::default()
    };
    let runtime_paths = test_runtime_paths();

    let settings =
        resolve_test_settings(&test_args(Some(1)), &app_config, &runtime_paths).expect("settings");

    assert_eq!(settings.failure_policy, TestFailurePolicy::SkipRemaining);
    assert!(settings.failure_policy.halts_after_failure());
    assert!(TestFailurePolicy::MarkFailed.halts_after_failure());
    assert!(!TestFailurePolicy::Continue.halts_after_failure());
}
