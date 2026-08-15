use super::super::NodeDedupKey;
use crate::model::Protocol;

#[test]
fn covers_all_protocol_variants() {
    let protocols = vec![
        (Protocol::Vless, "vless"),
        (Protocol::Vmess, "vmess"),
        (Protocol::Ss, "ss"),
        (Protocol::Trojan, "trojan"),
        (Protocol::Http, "http"),
        (Protocol::Socks5, "socks5"),
        (Protocol::Hy2, "hy2"),
    ];

    for (protocol, expected_name) in protocols {
        let key = NodeDedupKey {
            protocol: protocol.clone(),
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
            extensions: None,
        };

        let output = key.to_string();
        let expected = format!("protocol={}:{}", expected_name.len(), expected_name);
        assert!(
            output.contains(&expected),
            "Protocol {:?} should produce {}",
            protocol,
            expected
        );
    }
}

#[test]
fn produces_identical_keys_for_equivalent_configs() {
    let key1 = NodeDedupKey {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("test-uuid".to_string()),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("sni.example.com".to_string()),
        host: Some("host.example.com".to_string()),
        path: Some("/path".to_string()),
        extensions: None,
    };

    let key2 = key1.clone();

    assert_eq!(key1.to_string(), key2.to_string());
}

#[test]
fn distinguishes_different_network_types() {
    let base = NodeDedupKey {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: None,
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        extensions: None,
    };

    let ws_key = NodeDedupKey {
        network: "ws".to_string(),
        ..base.clone()
    };
    let grpc_key = NodeDedupKey {
        network: "grpc".to_string(),
        ..base.clone()
    };

    assert_ne!(base.to_string(), ws_key.to_string());
    assert_ne!(base.to_string(), grpc_key.to_string());
    assert_ne!(ws_key.to_string(), grpc_key.to_string());
}

#[test]
fn handles_all_optional_fields_present() {
    let key = NodeDedupKey {
        protocol: Protocol::Ss,
        address: "example.com".to_string(),
        port: 8388,
        username: Some("user".to_string()),
        uuid: Some("uuid".to_string()),
        password: Some("pass".to_string()),
        method: Some("aes-256-gcm".to_string()),
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("sni.example.com".to_string()),
        host: Some("host.example.com".to_string()),
        path: Some("/path".to_string()),
        extensions: Some(Default::default()),
    };

    let output = key.to_string();
    assert!(!output.contains("=-"), "No field should be None");
    assert!(output.contains("username=4:user"));
    assert!(output.contains("uuid=4:uuid"));
    assert!(output.contains("password=4:pass"));
    assert!(output.contains("method=11:aes-256-gcm"));
}

#[test]
fn handles_all_optional_fields_absent() {
    let key = NodeDedupKey {
        protocol: Protocol::Http,
        address: "example.com".to_string(),
        port: 80,
        username: None,
        uuid: None,
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        extensions: None,
    };

    let output = key.to_string();
    let none_count = output.matches("=-").count();
    assert_eq!(none_count, 9, "Should have 9 None fields");
}
