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
