use super::{
    SingboxDnsConfig, SingboxInbound, SingboxRouteList, SingboxRoutingOptions,
    generate_singbox_probe_config, generate_singbox_runtime_config,
    generate_singbox_runtime_config_with_dns,
};
use crate::model::{Node, Protocol};
use std::collections::BTreeMap;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

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
fn generates_runtime_dns_without_adding_it_to_probes() {
    let node = hy2_node(None);
    let dns = SingboxDnsConfig {
        servers: vec![
            serde_json::json!({
                "type": "udp",
                "tag": "xrat-dns-0",
                "server": "8.8.8.8",
                "server_port": 53,
            }),
            serde_json::json!({
                "type": "hosts",
                "tag": "xrat-dns-hosts",
                "predefined": {"example.test": "192.0.2.10"},
            }),
        ],
        rules: vec![
            serde_json::json!({
                "domain": ["example.test"],
                "action": "route",
                "server": "xrat-dns-hosts",
            }),
            serde_json::json!({
                "ip_accept_any": true,
                "action": "route",
                "server": "xrat-dns-hosts",
            }),
        ],
        final_server: "xrat-dns-0".to_string(),
        strategy: Some("ipv4_only".to_string()),
        disable_cache: Some(true),
    };

    let config =
        generate_singbox_runtime_config_with_dns(&node, Vec::new(), None, None, Some(&dns))
            .expect("runtime DNS should be attached");
    let value = serde_json::to_value(config).expect("config should serialize");
    assert_eq!(value["dns"]["final"], "xrat-dns-0");
    assert_eq!(value["dns"]["strategy"], "ipv4_only");
    assert_eq!(value["dns"]["disable_cache"], true);

    let probe = generate_singbox_probe_config(&node, 1080).unwrap();
    assert!(serde_json::to_value(probe).unwrap().get("dns").is_none());
}

#[test]
fn native_singbox_validator_accepts_generated_dns_config() {
    if Command::new("sing-box").arg("version").output().is_err() {
        return;
    }

    let node = hy2_node(None);
    let dns = SingboxDnsConfig {
        servers: vec![serde_json::json!({
            "type": "udp",
            "tag": "xrat-dns-0",
            "server": "8.8.8.8",
            "server_port": 53,
        })],
        rules: Vec::new(),
        final_server: "xrat-dns-0".to_string(),
        strategy: Some("ipv4_only".to_string()),
        disable_cache: Some(true),
    };
    let config =
        generate_singbox_runtime_config_with_dns(&node, Vec::new(), None, None, Some(&dns))
            .expect("runtime config should generate");
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(serde_json::to_string(&config).unwrap().as_bytes())
        .unwrap();
    file.flush().unwrap();

    let output = Command::new("sing-box")
        .args(["check", "-c"])
        .arg(file.path())
        .output()
        .expect("sing-box should start");
    assert!(
        output.status.success(),
        "sing-box rejected generated DNS config: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn generates_ordered_singbox_routing_with_proxy_fallback() {
    let node = hy2_node(None);
    let routing = SingboxRoutingOptions {
        direct: SingboxRouteList {
            domain: vec![
                "full:exact.example".to_string(),
                "domain:suffix.example".to_string(),
                "keyword:direct".to_string(),
                "regexp:^safe\\.example$".to_string(),
            ],
            ip: vec!["192.168.0.0/16".to_string()],
            ..Default::default()
        },
        block: SingboxRouteList {
            domain: vec!["domain:ads.example".to_string()],
            ip: vec!["203.0.113.0/24".to_string()],
            ..Default::default()
        },
    };

    let config = generate_singbox_runtime_config(&node, Vec::new(), None, Some(&routing))
        .expect("supported routing should generate");
    let value = serde_json::to_value(config).expect("config should serialize");

    assert_eq!(
        value["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|outbound| outbound["tag"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["proxy", "direct", "block"]
    );
    assert_eq!(value["route"]["final"], "proxy");
    assert_eq!(value["route"]["rules"][0]["outbound"], "direct");
    assert_eq!(value["route"]["rules"][0]["domain"][0], "exact.example");
    assert_eq!(
        value["route"]["rules"][0]["domain_suffix"][0],
        "suffix.example"
    );
    assert_eq!(value["route"]["rules"][0]["domain_keyword"][0], "direct");
    assert_eq!(
        value["route"]["rules"][0]["domain_regex"][0],
        "^safe\\.example$"
    );
    assert_eq!(value["route"]["rules"][1]["ip_cidr"][0], "192.168.0.0/16");
    assert_eq!(value["route"]["rules"][2]["outbound"], "block");
    assert_eq!(value["route"]["rules"][3]["outbound"], "block");
}

#[test]
fn rejects_singbox_geosite_and_geoip_until_rule_sets_are_supported() {
    let node = hy2_node(None);
    let routing = SingboxRoutingOptions {
        direct: SingboxRouteList {
            geosite: vec!["private".to_string()],
            ..Default::default()
        },
        block: SingboxRouteList::default(),
    };

    let error = generate_singbox_runtime_config(&node, Vec::new(), None, Some(&routing))
        .expect_err("geosite should require rule-set support");

    assert!(error.contains("routing.direct.geosite/geoip"));
    assert!(error.contains("rule-set support"));
}

#[test]
fn rejects_xray_only_singbox_domain_syntax() {
    let node = hy2_node(None);
    let routing = SingboxRoutingOptions {
        direct: SingboxRouteList::default(),
        block: SingboxRouteList {
            domain: vec!["ext:custom.dat:ads".to_string()],
            ..Default::default()
        },
    };

    let error = generate_singbox_runtime_config(&node, Vec::new(), None, Some(&routing))
        .expect_err("Xray-only syntax should be rejected");

    assert!(error.contains("routing.block.domain"));
    assert!(error.contains("not translatable"));
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
