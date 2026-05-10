use crate::config::xray::{DnsHostValue, FakeDnsObject, LogObject, XrayConfig};

#[test]
fn test_parse_policy_object() {
    let json = r#"{
        "policy": {
            "levels": {
                "0": {
                    "handshake": 4,
                    "connIdle": 300,
                    "statsUserUplink": true
                }
            },
            "system": {
                "statsInboundUplink": true,
                "statsInboundDownlink": true
            }
        },
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    let policy = config.policy.unwrap();
    assert!(policy.levels.is_some());
    assert!(policy.system.is_some());
}

#[test]
fn test_parse_features() {
    let json = r#"{
        "stats": {},
        "metrics": {
            "tag": "metrics",
            "listen": "127.0.0.1:9090"
        },
        "observatory": {
            "subjectSelector": ["outbound"],
            "probeUrl": "https://www.google.com/generate_204",
            "probeInterval": "1m"
        },
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    assert!(config.stats.is_some());
    assert!(config.metrics.is_some());
    assert!(config.observatory.is_some());
}

#[test]
fn test_serialize_to_json() {
    let config = XrayConfig {
        version: None,
        log: Some(LogObject {
            access: Some("/var/log/access.log".to_string()),
            error: None,
            loglevel: Some(crate::config::xray::shared::LogLevel::Info),
            dns_log: None,
            mask_address: None,
        }),
        api: None,
        dns: None,
        routing: None,
        policy: None,
        inbounds: Some(vec![]),
        outbounds: Some(vec![]),
        transport: None,
        stats: None,
        reverse: None,
        fakedns: None,
        metrics: None,
        observatory: None,
        burst_observatory: None,
    };

    let json = config.to_json().unwrap();
    assert!(json.contains("\"log\""));
    assert!(json.contains("\"access\""));
    assert!(json.contains("\"loglevel\""));
}

#[test]
fn test_port_value_parsing() {
    use crate::config::xray::shared::PortValue;

    let single: PortValue = serde_json::from_str("443").unwrap();
    assert!(matches!(single, PortValue::Single(443)));

    let range: PortValue = serde_json::from_str("\"1000-2000\"").unwrap();
    assert!(matches!(range, PortValue::Range(_)));
}

#[test]
fn test_dns_host_value_parsing() {
    let single: DnsHostValue = serde_json::from_str("\"127.0.0.1\"").unwrap();
    assert!(matches!(single, DnsHostValue::Single(_)));

    let multiple: DnsHostValue = serde_json::from_str("[\"127.0.0.1\", \"::1\"]").unwrap();
    assert!(matches!(multiple, DnsHostValue::Multiple(_)));
}

#[test]
fn test_fakedns_parsing() {
    let single: FakeDnsObject = serde_json::from_str(r#"{"ipPool": "198.18.0.0/15"}"#).unwrap();
    assert!(matches!(single, FakeDnsObject::Single(_)));

    let multiple: FakeDnsObject = serde_json::from_str(r#"[{"ipPool": "198.18.0.0/15"}]"#).unwrap();
    assert!(matches!(multiple, FakeDnsObject::Multiple(_)));
}
