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
