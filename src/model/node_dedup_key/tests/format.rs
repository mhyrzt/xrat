use super::super::NodeDedupKey;
use crate::model::Protocol;

#[test]
fn formats_as_versioned_length_prefixed_key() {
    let key = NodeDedupKey {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("uuid|123".to_string()),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("cdn.example.com".to_string()),
        host: Some("cdn.example.com".to_string()),
        path: Some("/ray".to_string()),
        extensions: None,
    };

    assert_eq!(
        key.to_string(),
        "v2|protocol=5:vless|address=11:example.com|port=3:443|username=-|uuid=8:uuid|123|password=-|method=-|network=2:ws|tls=3:tls|sni=15:cdn.example.com|host=15:cdn.example.com|path=4:/ray|extensions=-"
    );
}

#[test]
fn distinguishes_none_from_empty_string() {
    let none_key = NodeDedupKey {
        protocol: Protocol::Ss,
        address: "example.com".to_string(),
        port: 8388,
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
    let empty_key = NodeDedupKey {
        password: Some(String::new()),
        ..none_key.clone()
    };

    assert_ne!(none_key.to_string(), empty_key.to_string());
    assert!(empty_key.to_string().contains("|password=0:"));
}

#[test]
fn handles_unicode_multibyte_characters() {
    let key = NodeDedupKey {
        protocol: Protocol::Vless,
        address: "例え.jp".to_string(),
        port: 443,
        username: None,
        uuid: None,
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: Some("/路径".to_string()),
        extensions: None,
    };

    let output = key.to_string();
    // "例え.jp" has 5 chars (2 Japanese + 1 dot + 2 ASCII)
    // "/路径" has 3 chars (1 slash + 2 Chinese)
    assert!(
        output.contains("address=5:例え.jp"),
        "output was: {}",
        output
    );
    assert!(output.contains("path=3:/路径"), "output was: {}", output);
}

#[test]
fn handles_special_characters_in_fields() {
    let key = NodeDedupKey {
        protocol: Protocol::Vmess,
        address: "host=example.com".to_string(),
        port: 443,
        username: Some("user|name".to_string()),
        uuid: None,
        password: Some("pass=word|123".to_string()),
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        extensions: None,
    };

    let output = key.to_string();
    assert!(output.contains("address=16:host=example.com"));
    assert!(output.contains("username=9:user|name"));
    assert!(output.contains("password=13:pass=word|123"));
}

#[test]
fn handles_port_boundary_values() {
    let min_key = NodeDedupKey {
        protocol: Protocol::Trojan,
        address: "example.com".to_string(),
        port: 0,
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
    let max_key = NodeDedupKey {
        port: u16::MAX,
        ..min_key.clone()
    };

    assert!(min_key.to_string().contains("port=1:0"));
    assert!(max_key.to_string().contains("port=5:65535"));
    assert_ne!(min_key.to_string(), max_key.to_string());
}
