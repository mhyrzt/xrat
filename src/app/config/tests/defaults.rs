use crate::app::config::testing::TestFailurePolicy;
use crate::app::config::{AppConfig, ConnectionTestStage};

#[test]
fn parses_minimal_config_with_defaults() {
    let config: AppConfig = toml::from_str("").expect("empty config should use defaults");

    assert_eq!(config.runtime.engine, "xray");
    assert!(!config.runtime.rotation.enabled);
    assert_eq!(config.runtime.rotation.interval_secs, 1800);
    assert!(config.runtime.rotation.health_trigger_enabled);
    assert_eq!(config.runtime.rotation.cooldown_secs, 300);
    assert_eq!(config.runtime.socks.port, 1080);
    assert_eq!(config.testing.concurrency, 0);
    assert_eq!(
        config.testing.order,
        vec![
            ConnectionTestStage::Icmp,
            ConnectionTestStage::RealDelay,
            ConnectionTestStage::Download,
        ]
    );
    assert_eq!(config.testing.failure_policy, TestFailurePolicy::Continue);
    assert!(config.testing.icmp.enabled);
    assert_eq!(config.testing.icmp.attempts, 3);
    assert_eq!(config.testing.icmp.timeout, 3000);
    assert_eq!(
        config.testing.real_delay.url,
        crate::app::config::defaults::DEFAULT_REAL_DELAY_TEST_URL
    );
    assert!(!config.testing.geoip.enabled);
    assert_eq!(
        config.testing.geoip.country_path,
        std::path::PathBuf::from(crate::app::config::defaults::DEFAULT_TEST_GEOIP_COUNTRY_PATH)
    );
    assert_eq!(
        config.testing.geoip.city_path,
        std::path::PathBuf::from(crate::app::config::defaults::DEFAULT_TEST_GEOIP_CITY_PATH)
    );
    assert_eq!(
        config.testing.geoip.asn_path,
        std::path::PathBuf::from(crate::app::config::defaults::DEFAULT_TEST_GEOIP_ASN_PATH)
    );
    assert_eq!(
        config.parser.parse_mode,
        crate::xray::parsing::ParseMode::Strict
    );
    assert!(!config.server.enabled);
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.key, None);
}
