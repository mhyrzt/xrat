use crate::app::config::testing::TestFailurePolicy;
use crate::app::config::{
    AppConfig, ConnectionTestStage, DatabaseBackend, DnsHostValue, SecretString,
};

#[test]
fn parses_example_config() {
    let config: AppConfig =
        toml::from_str(include_str!("../../../../testdata/config.example.toml"))
            .expect("example config should parse");

    assert_eq!(config.paths.database.as_deref(), Some("db.sqlite".as_ref()));
    assert_eq!(config.database.backend, DatabaseBackend::Sqlite);
    assert_eq!(
        config.database.sqlite.path.as_deref(),
        Some("db.sqlite".as_ref())
    );
    assert_eq!(config.runtime.engine, "xray");
    assert!(config.runtime.rotation.enabled);
    assert_eq!(config.runtime.rotation.interval_secs, 1800);
    assert!(config.runtime.rotation.health_trigger_enabled);
    assert_eq!(config.runtime.rotation.cooldown_secs, 300);
    assert_eq!(
        config.runtime.rotation.test_stages,
        vec!["icmp".to_string(), "real_delay".to_string()]
    );
    assert_eq!(config.mmdb.dir, std::path::PathBuf::from("mmdb"));
    assert_eq!(config.runtime.socks.auth.username.as_deref(), Some("xrat"));
    assert_eq!(
        config.runtime.socks.auth.password,
        Some(SecretString::Env {
            env: "XRAT_SOCKS_PASSWORD".to_string()
        })
    );
    assert_eq!(
        config.runtime.shadowsocks.password,
        SecretString::Env {
            env: "XRAT_SHADOWSOCKS_PASSWORD".to_string()
        }
    );
    assert!(config.runtime.mux.enabled);
    assert_eq!(config.runtime.mux.concurrency, 4);
    assert_eq!(config.runtime.mux.xudp_concurrency, 16);
    assert_eq!(config.runtime.mux.xudp_proxy_udp443, "skip");
    assert!(config.runtime.fragment.enabled);
    assert_eq!(config.runtime.fragment.packets_mode, "tlshello");
    assert_eq!(config.runtime.fragment.packets, [1, 3]);
    assert_eq!(config.runtime.fragment.length, [100, 200]);
    assert_eq!(config.runtime.fragment.interval, [10, 20]);
    assert_eq!(config.runtime.network.interface, "eth0");
    assert_eq!(config.runtime.network.bind_address, "192.168.1.10");
    assert_eq!(config.runtime.network.mark, 255);
    assert_eq!(config.runtime.network.listen_interface, "");
    assert_eq!(config.routing.domain_strategy, "IPIfNonMatch");
    assert_eq!(config.geo.profiles.len(), 2);
    assert_eq!(config.dns.servers.len(), 5);
    assert_eq!(
        config.dns.hosts.get("domain:example.test"),
        Some(&DnsHostValue::One("127.0.0.1".to_string()))
    );
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
    assert!(config.testing.real_delay.enabled);
    assert!(config.testing.icmp.enabled);
    assert_eq!(config.testing.icmp.attempts, 3);
    assert!(!config.testing.download.enabled);
    assert!(config.testing.tcp.enabled);
    assert_eq!(config.testing.tcp.timeout, 5000);
    assert_eq!(
        config.parser.parse_mode,
        crate::xray::parsing::ParseMode::Strict
    );
    assert!(!config.server.enabled);
    assert_eq!(config.server.host, "127.0.0.1");
    assert_eq!(config.server.port, 8080);
    assert_eq!(
        config.server.key,
        Some(SecretString::Env {
            env: "XRAT_API_KEY".to_string()
        })
    );
    assert!(config.server.pac_enabled);
    assert_eq!(
        config.server.pac_allowed_hosts,
        vec!["localhost", "127.0.0.1", "::1"]
    );
}
