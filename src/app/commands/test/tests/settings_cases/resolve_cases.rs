use super::super::super::*;
use super::super::geoip_cases::{test_args, test_runtime_paths};

use crate::app::config::{AppConfig, TestingSettings};
use crate::cli::TestArgs;

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
    assert_eq!(settings.geoip_lookup.backend_name(), "mmdb");
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
    assert_eq!(settings.geoip_lookup.backend_name(), "mmdb");
}

#[test]
fn default_geoip_paths_resolve_from_runtime_root_mmdb_dir() {
    let app_config = AppConfig::default();
    let args = test_args(Some(1));
    let runtime_paths = crate::app::context::RuntimePaths {
        root_dir: "/tmp/xrat-root".into(),
        config_path: "/tmp/custom/config.toml".into(),
        ..test_runtime_paths()
    };

    let settings = resolve_test_settings(&args, &app_config, &runtime_paths).expect("settings");

    assert_eq!(
        settings.geoip_country_path,
        PathBuf::from("/tmp/xrat-root/mmdb/GeoLite2-Country.mmdb")
    );
    assert_eq!(
        settings.geoip_city_path,
        PathBuf::from("/tmp/xrat-root/mmdb/GeoLite2-City.mmdb")
    );
    assert_eq!(
        settings.geoip_asn_path,
        PathBuf::from("/tmp/xrat-root/mmdb/GeoLite2-ASN.mmdb")
    );
    assert_eq!(settings.geoip_lookup.backend_name(), "mmdb");
}
