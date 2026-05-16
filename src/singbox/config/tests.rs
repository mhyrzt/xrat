use super::generate_singbox_probe_config;
use crate::model::{Node, Protocol};
use std::collections::BTreeMap;

#[test]
fn generates_hy2_singbox_config_with_optional_fields() {
    let node = Node {
        protocol: Protocol::Hy2,
        address: "hy2.example.com".to_string(),
        port: 443,
        username: None,
        uuid: None,
        password: Some("secret".to_string()),
        method: None,
        network: "udp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("edge.example.com".to_string()),
        host: None,
        path: None,
        name: Some("hy2".to_string()),
        extensions: None,
        raw_config: "hy2://secret@hy2.example.com:443?sni=edge.example.com&insecure=1&alpn=h3,h2&obfs=salamander&obfs-password=pwd&upmbps=20&downmbps=80#hy2".to_string(),
    };

    let config = generate_singbox_probe_config(&node, 1080).expect("hy2 config should generate");
    let outbound = &config.outbounds[0];
    assert_eq!(outbound["type"], "hysteria2");
    assert_eq!(outbound["tls"]["insecure"], true);
    assert_eq!(outbound["tls"]["alpn"], serde_json::json!(["h3", "h2"]));
    assert_eq!(outbound["obfs"]["type"], "salamander");
    assert_eq!(outbound["obfs"]["password"], "pwd");
    assert_eq!(outbound["up_mbps"], 20);
    assert_eq!(outbound["down_mbps"], 80);
}

#[test]
fn prefers_protocol_extensions_when_present() {
    let mut extensions = BTreeMap::new();
    extensions.insert("insecure".to_string(), "1".to_string());
    extensions.insert("obfs".to_string(), "salamander".to_string());
    extensions.insert("obfs-password".to_string(), "pwd".to_string());

    let node = Node {
        protocol: Protocol::Hy2,
        address: "hy2.example.com".to_string(),
        port: 443,
        username: None,
        uuid: None,
        password: Some("secret".to_string()),
        method: None,
        network: "udp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("edge.example.com".to_string()),
        host: None,
        path: None,
        name: Some("hy2".to_string()),
        extensions: Some(extensions),
        raw_config: "hy2://secret@hy2.example.com:443#hy2".to_string(),
    };

    let config = generate_singbox_probe_config(&node, 1080).expect("hy2 config should generate");
    let outbound = &config.outbounds[0];
    assert_eq!(outbound["tls"]["insecure"], true);
    assert_eq!(outbound["obfs"]["password"], "pwd");
}
