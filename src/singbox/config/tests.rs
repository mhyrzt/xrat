use super::{SingboxInbound, generate_singbox_probe_config, generate_singbox_runtime_config};
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
fn generates_hy2_runtime_config_with_multiple_local_inbounds() {
    let node = hy2_node(None);
    let config = generate_singbox_runtime_config(
        &node,
        vec![
            SingboxInbound {
                kind: "socks".to_string(),
                tag: "socks-in".to_string(),
                listen: "127.0.0.1".to_string(),
                listen_port: 1080,
                network: Some("udp".to_string()),
                method: None,
                password: None,
                users: None,
            },
            SingboxInbound {
                kind: "http".to_string(),
                tag: "http-in".to_string(),
                listen: "127.0.0.1".to_string(),
                listen_port: 8080,
                network: None,
                method: None,
                password: None,
                users: None,
            },
        ],
        None,
    )
    .expect("hy2 runtime config should generate");

    let value = serde_json::to_value(config).expect("config should serialize");
    assert_eq!(value["log"]["timestamp"], true);
    assert_eq!(value["inbounds"][0]["type"], "socks");
    assert_eq!(value["inbounds"][0]["listen_port"], 1080);
    assert_eq!(value["inbounds"][0]["network"], "udp");
    assert_eq!(value["inbounds"][1]["type"], "http");
    assert_eq!(value["outbounds"][0]["type"], "hysteria2");
    assert!(value.get("experimental").is_none());
}

#[test]
fn prefers_protocol_extensions_when_present() {
    let mut extensions = BTreeMap::new();
    extensions.insert("insecure".to_string(), serde_json::json!("1"));
    extensions.insert("obfs".to_string(), serde_json::json!("salamander"));
    extensions.insert("obfs-password".to_string(), serde_json::json!("pwd"));

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

fn hy2_node(extensions: Option<BTreeMap<String, serde_json::Value>>) -> Node {
    Node {
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
        extensions,
        raw_config: "hy2://secret@hy2.example.com:443?sni=edge.example.com#hy2".to_string(),
    }
}
