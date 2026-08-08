use crate::app::config::testing::TestFailurePolicy;
use crate::app::config::{AppConfig, ConnectionTestStage};

#[test]
fn parses_minimal_config_with_defaults() {
    let config: AppConfig = toml::from_str("").expect("empty config should use defaults");

    assert_eq!(config.runtime.engine, "xray");
    assert!(config.runtime.rotation.enabled);
    assert_eq!(config.runtime.rotation.interval_secs, 1800);
    assert!(config.runtime.rotation.health_trigger_enabled);
    assert_eq!(config.runtime.rotation.cooldown_secs, 300);
    assert_eq!(
        config.runtime.rotation.test_stages,
        vec!["icmp".to_string(), "real_delay".to_string()]
    );
    assert_eq!(config.runtime.socks.port, 18200);
    assert!(!config.runtime.mux.enabled);
    assert_eq!(config.runtime.mux.concurrency, 8);
    assert_eq!(config.runtime.mux.xudp_concurrency, 0);
    assert_eq!(config.runtime.mux.xudp_proxy_udp443, "reject");
    assert!(!config.runtime.fragment.enabled);
    assert_eq!(config.runtime.fragment.packets_mode, "tlshello");
    assert_eq!(config.runtime.fragment.packets, [1, 3]);
    assert_eq!(config.runtime.fragment.length, [100, 200]);
    assert_eq!(config.runtime.fragment.interval, [10, 20]);
    assert_eq!(config.runtime.network.interface, "");
    assert_eq!(config.runtime.network.bind_address, "");
    assert_eq!(config.runtime.network.mark, 0);
    assert_eq!(config.runtime.network.listen_interface, "");
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
    assert_eq!(config.mmdb.dir, std::path::PathBuf::from("mmdb"));
    assert_eq!(
        config.mmdb.download_url,
        crate::app::config::defaults::DEFAULT_MMDB_DOWNLOAD_URL
    );
    assert_eq!(
        config.mmdb.timeout_secs,
        crate::app::config::defaults::DEFAULT_MMDB_TIMEOUT_SECS
    );
    assert_eq!(
        config.testing.real_delay.url,
        crate::app::config::defaults::DEFAULT_REAL_DELAY_TEST_URL
    );
    assert!(config.testing.real_delay.accepted_status_codes.is_none());
    assert!(config.testing.real_delay.accepted_status_ranges.is_none());
    assert!(config.testing.real_delay.follow_redirects);
    assert!(!config.testing.geoip.enabled);
    assert_eq!(
        config.testing.geoip.backend,
        crate::app::config::GeoIpBackend::Mmdb
    );
    assert_eq!(
        config.testing.geoip.fallback,
        crate::app::config::GeoIpBackend::None
    );
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
        config.testing.geoip.remote.provider,
        crate::app::config::GeoIpRemoteProvider::IpWhois
    );
    assert!(config.testing.geoip.cache.enabled);
    assert_eq!(config.testing.geoip.cache.ttl_secs, 86_400);
    assert_eq!(config.testing.geoip.cache.max_entries, 10_000);
    assert_eq!(
        config.parser.parse_mode,
        crate::xray::parsing::ParseMode::Strict
    );
    assert!(!config.server.enabled);
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 18203);
    assert_eq!(config.server.key, None);
    assert!(config.server.pac_enabled);
    assert_eq!(
        config.server.pac_allowed_hosts,
        vec!["localhost", "127.0.0.1", "::1"]
    );
}

#[test]
fn parses_tcp_as_standalone_test_stage() {
    let config: AppConfig = toml::from_str(
        r#"
        [testing]
        order = ["icmp", "tcp"]
        "#,
    )
    .expect("tcp should be an accepted stage in [testing].order");

    assert_eq!(
        config.testing.order,
        vec![ConnectionTestStage::Icmp, ConnectionTestStage::Tcp]
    );
}
