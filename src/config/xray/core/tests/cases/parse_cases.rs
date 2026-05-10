use crate::config::xray::ParseMode;
use crate::config::xray::XrayConfig;

#[test]
fn test_parse_minimal_config_loose() {
    let json = r#"{
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    assert!(config.inbounds.is_some());
    assert!(config.outbounds.is_some());
    assert!(config.log.is_none());
}

#[test]
fn test_parse_minimal_config_strict() {
    let json = r#"{
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_strict(json).unwrap();
    assert!(config.inbounds.is_some());
    assert!(config.outbounds.is_some());
}

#[test]
fn test_loose_mode_allows_unknown_fields() {
    let json = r#"{
        "inbounds": [],
        "outbounds": [],
        "unknownField": "should be ignored",
        "anotherUnknown": 123
    }"#;

    let result = XrayConfig::from_json_loose(json);
    assert!(result.is_ok(), "Loose mode should allow unknown fields");
}

#[test]
fn test_strict_mode_rejects_unknown_fields() {
    let json = r#"{
        "inbounds": [],
        "outbounds": [],
        "unknownField": "should cause error"
    }"#;

    let result = XrayConfig::from_json_strict(json);
    assert!(result.is_err(), "Strict mode should reject unknown fields");
}

#[test]
fn test_parse_mode_controls_unknown_fields() {
    let json = r#"{
        "inbounds": [],
        "outbounds": [],
        "unknownField": "allowed outside strict mode"
    }"#;

    assert!(XrayConfig::from_json_with_mode(json, ParseMode::Strict).is_err());
    assert!(XrayConfig::from_json_with_mode(json, ParseMode::Lenient).is_ok());
    assert!(XrayConfig::from_json_with_mode(json, ParseMode::Auto).is_ok());
}

#[test]
fn test_parse_log_object() {
    let json = r#"{
        "log": {
            "access": "/var/log/xray/access.log",
            "error": "/var/log/xray/error.log",
            "loglevel": "warning",
            "dnsLog": true
        },
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    let log = config.log.unwrap();
    assert_eq!(log.access.as_deref(), Some("/var/log/xray/access.log"));
    assert_eq!(log.error.as_deref(), Some("/var/log/xray/error.log"));
    assert_eq!(
        log.loglevel,
        Some(crate::config::xray::shared::LogLevel::Warning)
    );
    assert_eq!(log.dns_log, Some(true));
}

#[test]
fn test_parse_api_object() {
    let json = r#"{
        "api": {
            "tag": "api",
            "listen": "127.0.0.1:8080",
            "services": ["HandlerService", "StatsService"]
        },
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    let api = config.api.unwrap();
    assert_eq!(api.tag, "api");
    assert_eq!(api.listen.as_deref(), Some("127.0.0.1:8080"));
    assert_eq!(api.services.len(), 2);
}

#[test]
fn test_parse_dns_object_with_simple_servers() {
    let json = r#"{
        "dns": {
            "servers": ["8.8.8.8", "1.1.1.1"],
            "queryStrategy": "UseIPv4"
        },
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    let dns = config.dns.unwrap();
    assert_eq!(dns.servers.as_ref().unwrap().len(), 2);
    assert_eq!(
        dns.query_strategy,
        Some(crate::config::xray::shared::QueryStrategy::UseIPv4)
    );
}

#[test]
fn test_parse_dns_object_with_full_servers() {
    let json = r#"{
        "dns": {
            "servers": [
                {
                    "address": "8.8.8.8",
                    "port": 53,
                    "domains": ["google.com"],
                    "skipFallback": true
                }
            ]
        },
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    let dns = config.dns.unwrap();
    assert_eq!(dns.servers.as_ref().unwrap().len(), 1);
}

#[test]
fn test_parse_routing_object() {
    let json = r#"{
        "routing": {
            "domainStrategy": "AsIs",
            "rules": [
                {
                    "domain": ["google.com"],
                    "outboundTag": "direct"
                }
            ]
        },
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    let routing = config.routing.unwrap();
    assert_eq!(routing.domain_strategy.as_deref(), Some("AsIs"));
    assert_eq!(routing.rules.as_ref().unwrap().len(), 1);
}
