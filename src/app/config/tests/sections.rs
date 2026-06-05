use crate::app::config::{AppConfig, SecretString};

#[test]
fn parses_server_settings() {
    let config: AppConfig = toml::from_str(
        r#"
[server]
enabled = true
host = "0.0.0.0"
port = 9090
key = "local-secret"
"#,
    )
    .expect("config should parse");

    assert!(config.server.enabled);
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 9090);
    assert_eq!(
        config.server.key,
        Some(SecretString::Literal("local-secret".to_string()))
    );
}

#[test]
fn subscription_settings_default_to_disabled_daily() {
    let config = AppConfig::default();
    assert!(!config.subscriptions.auto_refresh);
    assert_eq!(config.subscriptions.refresh_interval_hours, 24);
    assert_eq!(config.subscriptions.refresh_interval_secs(), 24 * 3600);
}

#[test]
fn parses_subscription_settings() {
    let config: AppConfig = toml::from_str(
        r#"
[subscriptions]
auto_refresh = true
refresh_interval_hours = 6
"#,
    )
    .expect("config should parse");

    assert!(config.subscriptions.auto_refresh);
    assert_eq!(config.subscriptions.refresh_interval_hours, 6);
    assert_eq!(config.subscriptions.refresh_interval_secs(), 6 * 3600);
}

#[test]
fn subscription_refresh_interval_is_clamped_to_at_least_one_hour() {
    let config: AppConfig =
        toml::from_str("[subscriptions]\nrefresh_interval_hours = 0\n").expect("parse");
    assert_eq!(config.subscriptions.refresh_interval_secs(), 3600);
}

#[test]
fn parses_parser_settings() {
    let config: AppConfig =
        toml::from_str("[parser]\nparse_mode = \"lenient\"\n").expect("config should parse");

    assert_eq!(
        config.parser.parse_mode,
        crate::xray::parsing::ParseMode::Lenient
    );
}

#[test]
fn parses_mmdb_settings() {
    let config: AppConfig = toml::from_str(
        r#"
[mmdb]
dir = "assets/mmdb"
download_url = "https://mirror.example.com/{edition}.mmdb"
timeout_secs = 30
default_editions = ["country", "asn"]
auto_update = true
update_interval_hours = 24
"#,
    )
    .expect("config should parse");

    assert_eq!(config.mmdb.dir, std::path::PathBuf::from("assets/mmdb"));
    assert_eq!(
        config.mmdb.download_url,
        "https://mirror.example.com/{edition}.mmdb"
    );
    assert_eq!(config.mmdb.timeout_secs, 30);
    assert_eq!(config.mmdb.default_editions, vec!["country", "asn"]);
    assert!(config.mmdb.auto_update);
    assert_eq!(config.mmdb.update_interval_hours, 24);
}

#[test]
fn parses_geoip_backend_settings() {
    let config: AppConfig = toml::from_str(
        r#"
[testing.geoip]
enabled = true
backend = "chain"
fallback = "ipwhois"
country_path = "custom/country.mmdb"
city_path = "custom/city.mmdb"
asn_path = "custom/asn.mmdb"

[testing.geoip.remote]
provider = "ip-api"
endpoint = "https://geo.example.test/json"
timeout_ms = 9000
api_key = "reserved"
rate_limit_per_minute = 15

[testing.geoip.cache]
enabled = false
ttl_secs = 60
max_entries = 100
"#,
    )
    .expect("config should parse");

    assert_eq!(
        config.testing.geoip.backend,
        crate::app::config::GeoIpBackend::Chain
    );
    assert_eq!(
        config.testing.geoip.fallback,
        crate::app::config::GeoIpBackend::IpWhois
    );
    assert_eq!(
        config.testing.geoip.remote.provider,
        crate::app::config::GeoIpRemoteProvider::IpApi
    );
    assert_eq!(
        config.testing.geoip.remote.endpoint,
        "https://geo.example.test/json"
    );
    assert_eq!(config.testing.geoip.remote.timeout_ms, 9000);
    assert_eq!(config.testing.geoip.remote.api_key, "reserved");
    assert_eq!(config.testing.geoip.remote.rate_limit_per_minute, 15);
    assert!(!config.testing.geoip.cache.enabled);
    assert_eq!(config.testing.geoip.cache.ttl_secs, 60);
    assert_eq!(config.testing.geoip.cache.max_entries, 100);
}
