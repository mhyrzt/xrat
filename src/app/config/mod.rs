use std::path::{Path, PathBuf};

use serde::Deserialize;

mod database;
pub(crate) mod defaults;
mod dns;
mod geo;
mod parser;
mod paths;
mod routing;
mod runtime;
mod secret;
mod testing;

pub use database::{
    DatabaseBackend, DatabaseSettings, PostgresDatabaseSettings, SqliteDatabaseSettings,
};
pub use dns::{DnsHostValue, DnsSettings};
pub use geo::{GeoProfile, GeoSettings};
pub use parser::ParserSettings;
pub use paths::PathSettings;
pub use routing::{RouteList, RoutingSettings};
pub use runtime::{
    AuthSettings, HttpSettings, LogSettings, RuntimeSettings, ShadowsocksSettings,
    SniffingSettings, SocksSettings,
};
pub use secret::{SecretError, SecretString};
pub use testing::{
    ConnectionTestStage, DownloadTestSettings, IcmpTestSettings, RealDelayTestSettings,
    TcpTestSettings, TestFailurePolicy, TestingSettings,
};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AppConfig {
    pub paths: PathSettings,
    pub database: DatabaseSettings,
    pub runtime: RuntimeSettings,
    pub routing: RoutingSettings,
    pub geo: GeoSettings,
    pub dns: DnsSettings,
    pub parser: ParserSettings,
    pub testing: TestingSettings,
}

pub fn load(config_path: &Path) -> crate::app::Result<AppConfig> {
    let contents = std::fs::read_to_string(config_path)?;
    let config = toml::from_str(&contents)?;

    Ok(config)
}

pub fn resolve_config_path(base_path: &Path, configured_path: &Path) -> PathBuf {
    if configured_path.is_absolute() {
        return configured_path.to_path_buf();
    }

    base_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join(configured_path)
}

#[cfg(test)]
mod tests {
    use super::{
        AppConfig, ConnectionTestStage, DatabaseBackend, DnsHostValue, SecretString,
        TestFailurePolicy, load, resolve_config_path,
    };

    #[test]
    fn parses_minimal_config_with_defaults() {
        let config: AppConfig = toml::from_str("").expect("empty config should use defaults");

        assert_eq!(config.runtime.engine, "xray");
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
        assert_eq!(
            config.parser.parse_mode,
            crate::config::xray::ParseMode::Strict
        );
    }

    #[test]
    fn parses_example_config() {
        let config: AppConfig =
            toml::from_str(include_str!("../../../docs/plan/config.example.toml"))
                .expect("example config should parse");

        assert_eq!(config.paths.database.as_deref(), Some("db.sqlite".as_ref()));
        assert_eq!(config.database.backend, DatabaseBackend::Sqlite);
        assert_eq!(
            config.database.sqlite.path.as_deref(),
            Some("db.sqlite".as_ref())
        );
        assert_eq!(config.runtime.engine, "xray");
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
            crate::config::xray::ParseMode::Strict
        );
    }

    #[test]
    fn parses_parser_settings() {
        let config: AppConfig =
            toml::from_str("[parser]\nparse_mode = \"lenient\"\n").expect("config should parse");

        assert_eq!(
            config.parser.parse_mode,
            crate::config::xray::ParseMode::Lenient
        );
    }

    #[test]
    fn resolves_relative_paths_from_config_directory() {
        let resolved = resolve_config_path("/tmp/xrat/config.toml".as_ref(), "db.sqlite".as_ref());

        assert_eq!(resolved, std::path::PathBuf::from("/tmp/xrat/db.sqlite"));
    }

    #[test]
    fn loads_config_file() {
        let root_dir = std::env::temp_dir().join(format!(
            "xrat-app-config-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        let config_path = root_dir.join("config.toml");
        std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
        std::fs::write(&config_path, "[runtime.socks]\nport = 1090\n")
            .expect("config should be written");

        let config = load(&config_path).expect("config should load");

        assert_eq!(config.runtime.socks.port, 1090);

        let _ = std::fs::remove_file(config_path);
        let _ = std::fs::remove_dir(root_dir);
    }
}
