use super::{
    SingboxDnsConfig, SingboxInbound, SingboxRouteList, SingboxRoutingOptions, SingboxTunOptions,
    generate_singbox_probe_config, generate_singbox_runtime_config,
    generate_singbox_runtime_config_with_dns,
};
use std::collections::BTreeMap;
use xrat_model::{Node, Protocol};

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
    assert_eq!(outbound["network"], "udp");
}

#[test]
fn generates_vless_tls_websocket_without_xray_fields() {
    let mut node = hy2_node(None);
    node.protocol = Protocol::Vless;
    node.uuid = Some("00000000-0000-0000-0000-000000000001".to_string());
    node.password = None;
    node.network = "ws".to_string();
    node.host = Some("front.example.com".to_string());
    node.path = Some("/stream".to_string());
    node.extensions = Some(BTreeMap::from([
        ("insecure".to_string(), serde_json::json!("1")),
        ("alpn".to_string(), serde_json::json!("h2,http/1.1")),
        (
            "cs".to_string(),
            serde_json::json!(
                "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256:TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"
            ),
        ),
    ]));

    let config = generate_singbox_probe_config(&node, 1080).unwrap();
    let outbound = &config.outbounds[0];
    assert_eq!(outbound["type"], "vless");
    assert_eq!(outbound["uuid"], "00000000-0000-0000-0000-000000000001");
    assert_eq!(outbound["tls"]["insecure"], true);
    assert_eq!(
        outbound["tls"]["alpn"],
        serde_json::json!(["h2", "http/1.1"])
    );
    assert_eq!(
        outbound["tls"]["cipher_suites"],
        serde_json::json!([
            "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
            "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256"
        ])
    );
    assert_eq!(outbound["transport"]["type"], "ws");
    assert_eq!(outbound["transport"]["path"], "/stream");
    assert_eq!(
        outbound["transport"]["headers"]["Host"],
        "front.example.com"
    );
    assert!(outbound.get("streamSettings").is_none());
}

#[test]
fn rejects_incomplete_vless_reality() {
    let mut node = hy2_node(None);
    node.protocol = Protocol::Vless;
    node.uuid = Some("00000000-0000-0000-0000-000000000001".to_string());
    node.password = None;
    node.tls = Some("reality".to_string());
    node.network = "tcp".to_string();
    assert!(
        generate_singbox_probe_config(&node, 1080)
            .unwrap_err()
            .contains("pbk/password public key")
    );
}

#[test]
fn rejects_invalid_managed_inbound_settings() {
    assert!(
        SingboxInbound::shadowsocks(
            "shadowsocks-in",
            "127.0.0.1",
            8388,
            "quic",
            "aes-128-gcm",
            "secret",
        )
        .unwrap_err()
        .contains("must be tcp or udp")
    );
    assert!(
        SingboxInbound::shadowsocks(
            "shadowsocks-in",
            "127.0.0.1",
            8388,
            "tcp",
            "chacha20-poly1305",
            "secret",
        )
        .unwrap_err()
        .contains("unsupported Shadowsocks inbound method")
    );
    assert!(
        SingboxInbound::shadowsocks(
            "shadowsocks-in",
            "127.0.0.1",
            8388,
            "tcp",
            "aes-128-gcm",
            "",
        )
        .unwrap_err()
        .contains("requires a password")
    );
}

fn tun_options() -> SingboxTunOptions {
    SingboxTunOptions {
        tag: "tun-in".to_string(),
        interface_name: "xrat0".to_string(),
        address: vec!["172.19.0.1/30".to_string()],
        mtu: 1500,
        stack: "system".to_string(),
        auto_route: true,
        strict_route: false,
        route_exclude_address: Vec::new(),
    }
}

#[test]
fn generates_tun_inbound_and_route() {
    let mut config = generate_singbox_runtime_config(
        &hy2_node(None),
        vec![
            SingboxInbound::socks("socks-in", "127.0.0.1", 1080, None),
            SingboxInbound::tun(tun_options()).expect("tun inbound should build"),
        ],
        None,
        None,
    )
    .expect("config should generate");
    config.enable_tun_route();
    let value = serde_json::to_value(&config).expect("config should serialize");

    let tun = &value["inbounds"][1];
    assert_eq!(tun["type"], "tun");
    assert_eq!(tun["interface_name"], "xrat0");
    assert_eq!(tun["address"], serde_json::json!(["172.19.0.1/30"]));
    assert_eq!(tun["auto_route"], true);
    assert_eq!(tun["strict_route"], false);
    assert_eq!(value["route"]["auto_detect_interface"], true);
    assert_eq!(value["route"]["rules"][0]["action"], "sniff");
    assert!(
        value["route"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|rule| rule["ip_is_private"] == true && rule["outbound"] == "direct")
    );
}

#[test]
fn serializes_tun_route_exclusions() {
    let mut options = tun_options();
    options.route_exclude_address = vec!["192.168.0.0/16".to_string()];
    let inbound = SingboxInbound::tun(options).expect("tun inbound should build");
    let value = serde_json::to_value(inbound).expect("inbound should serialize");
    assert_eq!(
        value["route_exclude_address"],
        serde_json::json!(["192.168.0.0/16"])
    );
}

#[test]
fn rejects_invalid_tun_options() {
    let mut options = tun_options();
    options.interface_name = "   ".to_string();
    assert!(
        SingboxInbound::tun(options)
            .unwrap_err()
            .contains("interface_name")
    );

    let mut options = tun_options();
    options.stack = "wireguard".to_string();
    assert!(SingboxInbound::tun(options).unwrap_err().contains("stack"));

    let mut options = tun_options();
    options.address = vec!["not-a-cidr".to_string()];
    assert!(SingboxInbound::tun(options).unwrap_err().contains("CIDR"));

    let mut options = tun_options();
    options.mtu = 100;
    assert!(SingboxInbound::tun(options).unwrap_err().contains("mtu"));

    let mut options = tun_options();
    options.route_exclude_address = vec!["10.0.0.0/33".to_string()];
    assert!(
        SingboxInbound::tun(options)
            .unwrap_err()
            .contains("route_exclude_address")
    );
}

#[test]
fn rejects_lossy_hy2_link_fields_before_generation() {
    let cases = [
        (
            "hy2://@hy2.example.com:443?sni=edge.example.com",
            "authentication password",
        ),
        (
            "hy2://secret@hy2.example.com:443?obfs=gecko",
            "only salamander",
        ),
        (
            "hy2://secret@hy2.example.com:443?upmbps=fast",
            "up_mbps must be an unsigned",
        ),
        (
            "hy2://secret@hy2.example.com:443?realm=unsupported",
            "option \"realm\" is not supported",
        ),
    ];

    for (raw_config, expected) in cases {
        let mut node = hy2_node(None);
        node.raw_config = raw_config.to_string();
        if raw_config.starts_with("hy2://@") {
            node.password = None;
        }
        let error = generate_singbox_probe_config(&node, 1080)
            .expect_err("lossy Hysteria2 input must be rejected");
        assert!(
            error.contains(expected),
            "expected {expected:?} in {error:?}"
        );
    }
}

#[test]
fn generates_hy2_runtime_config_with_multiple_local_inbounds() {
    let node = hy2_node(None);
    let config = generate_singbox_runtime_config(
        &node,
        vec![
            SingboxInbound::socks("socks-in", "127.0.0.1", 1080, None),
            SingboxInbound::http("http-in", "127.0.0.1", 8080),
        ],
        None,
        None,
    )
    .expect("hy2 runtime config should generate");

    let value = serde_json::to_value(config).expect("config should serialize");
    assert_eq!(value["log"]["timestamp"], true);
    assert_eq!(value["inbounds"][0]["type"], "socks");
    assert_eq!(value["inbounds"][0]["listen_port"], 1080);
    assert!(value["inbounds"][0].get("network").is_none());
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
        reverse_mapping: None,
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
fn generates_remote_rule_sets_for_geosite_and_geoip() {
    let node = hy2_node(None);
    let routing = SingboxRoutingOptions {
        direct: SingboxRouteList {
            geosite: vec!["private".to_string()],
            ..Default::default()
        },
        block: SingboxRouteList {
            geosite: vec!["category-ads-all".to_string()],
            geoip: vec!["cn".to_string()],
            ..Default::default()
        },
    };

    let config = generate_singbox_runtime_config(&node, Vec::new(), None, Some(&routing))
        .expect("remote rule-sets should generate");
    let value = serde_json::to_value(config).expect("config should serialize");

    let rule_sets = value["route"]["rule_set"].as_array().unwrap();
    assert_eq!(rule_sets.len(), 3);
    assert_eq!(rule_sets[0]["type"], "remote");
    assert_eq!(rule_sets[0]["format"], "binary");
    assert_eq!(
        rule_sets[0]["url"],
        "https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-private.srs"
    );
    assert_eq!(rule_sets[2]["tag"], "geoip-cn");
    assert_eq!(
        rule_sets[2]["url"],
        "https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-cn.srs"
    );
    assert_eq!(value["route"]["rules"][0]["rule_set"][0], "geosite-private");
    assert_eq!(value["route"]["rules"][0]["outbound"], "direct");
    assert_eq!(
        value["route"]["rules"][1]["rule_set"][0],
        "geosite-category-ads-all"
    );
    assert_eq!(value["route"]["rules"][1]["rule_set"][1], "geoip-cn");
    assert_eq!(value["route"]["rules"][1]["outbound"], "block");
}

#[test]
fn rejects_invalid_rule_set_category_names() {
    let node = hy2_node(None);
    let routing = SingboxRoutingOptions {
        direct: SingboxRouteList {
            geosite: vec!["../escape".to_string()],
            ..Default::default()
        },
        block: SingboxRouteList::default(),
    };

    let error = generate_singbox_runtime_config(&node, Vec::new(), None, Some(&routing))
        .expect_err("invalid rule-set names must fail");
    assert!(error.contains("routing.direct.geosite"));
    assert!(error.contains("not a valid rule-set category name"));
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

#[test]
fn rejects_unrepresentable_restored_hy2_extensions() {
    let mut extensions = BTreeMap::new();
    extensions.insert("upmbps".to_string(), serde_json::json!({"value": 20}));
    let error = generate_singbox_probe_config(&hy2_node(Some(extensions)), 1080)
        .expect_err("non-scalar restored extension must fail");
    assert!(error.contains("upmbps"));
    assert!(error.contains("string, boolean, or number"));
}

#[test]
fn generates_vless_plain_and_reality_variants() {
    let mut node = hy2_node(None);
    node.protocol = Protocol::Vless;
    node.uuid = Some("00000000-0000-0000-0000-000000000001".to_string());
    node.password = None;
    node.network = "tcp".to_string();
    node.tls = None;
    node.sni = None;

    let plain = generate_singbox_probe_config(&node, 1080).unwrap();
    assert!(plain.outbounds[0].get("tls").is_none());
    assert!(plain.outbounds[0].get("transport").is_none());

    node.tls = Some("reality".to_string());
    node.extensions = Some(BTreeMap::from([
        ("pbk".to_string(), serde_json::json!("public-key")),
        ("sid".to_string(), serde_json::json!("0123456789abcdef")),
        ("fp".to_string(), serde_json::json!("chrome")),
    ]));
    let reality = generate_singbox_probe_config(&node, 1080).unwrap();
    assert_eq!(
        reality.outbounds[0]["tls"]["reality"]["public_key"],
        "public-key"
    );
    assert_eq!(
        reality.outbounds[0]["tls"]["reality"]["short_id"],
        "0123456789abcdef"
    );
    assert_eq!(reality.outbounds[0]["tls"]["utls"]["fingerprint"], "chrome");

    node.extensions = Some(BTreeMap::from([
        ("pbk".to_string(), serde_json::json!("public-key")),
        ("sid".to_string(), serde_json::json!("01")),
    ]));
    let default_utls = generate_singbox_probe_config(&node, 1080).unwrap();
    assert_eq!(
        default_utls.outbounds[0]["tls"]["utls"],
        serde_json::json!({"enabled": true, "fingerprint": "chrome"})
    );
}

#[test]
fn rejects_invalid_vless_flow_fingerprint_and_short_id() {
    let base = || {
        let mut node = hy2_node(None);
        node.protocol = Protocol::Vless;
        node.uuid = Some("00000000-0000-0000-0000-000000000001".to_string());
        node.password = None;
        node.network = "tcp".to_string();
        node
    };

    let mut flow_over_ws = base();
    flow_over_ws.network = "ws".to_string();
    flow_over_ws.extensions = Some(BTreeMap::from([(
        "flow".to_string(),
        serde_json::json!("xtls-rprx-vision"),
    )]));
    assert!(
        generate_singbox_probe_config(&flow_over_ws, 1080)
            .unwrap_err()
            .contains("requires xtls-rprx-vision over direct TLS")
    );

    let mut bad_fingerprint = base();
    bad_fingerprint.tls = Some("tls".to_string());
    bad_fingerprint.extensions = Some(BTreeMap::from([(
        "fp".to_string(),
        serde_json::json!("not-a-fingerprint"),
    )]));
    assert!(
        generate_singbox_probe_config(&bad_fingerprint, 1080)
            .unwrap_err()
            .contains("unsupported sing-box uTLS fingerprint")
    );

    let mut bad_short_id = base();
    bad_short_id.tls = Some("reality".to_string());
    bad_short_id.extensions = Some(BTreeMap::from([
        ("pbk".to_string(), serde_json::json!("public-key")),
        ("sid".to_string(), serde_json::json!("not-hex!!")),
    ]));
    assert!(
        generate_singbox_probe_config(&bad_short_id, 1080)
            .unwrap_err()
            .contains("short id must be 0-16 hex digits")
    );
}

#[test]
fn generates_vmess_security_alter_id_and_packet_encoding() {
    let mut node = hy2_node(None);
    node.protocol = Protocol::Vmess;
    node.uuid = Some("00000000-0000-0000-0000-000000000001".to_string());
    node.password = None;
    node.network = "grpc".to_string();
    node.path = None;
    node.extensions = Some(BTreeMap::from([
        ("aid".to_string(), serde_json::json!(0)),
        ("scy".to_string(), serde_json::json!("auto")),
        ("packet_encoding".to_string(), serde_json::json!("xudp")),
        ("serviceName".to_string(), serde_json::json!("TunService")),
    ]));

    let config = generate_singbox_probe_config(&node, 1080).unwrap();
    let outbound = &config.outbounds[0];
    assert_eq!(outbound["security"], "auto");
    assert_eq!(outbound["alter_id"], 0);
    assert_eq!(outbound["packet_encoding"], "xudp");
    assert_eq!(outbound["transport"]["type"], "grpc");
    assert_eq!(outbound["transport"]["service_name"], "TunService");
}

#[test]
fn rejects_vmess_legacy_cipher_and_unknown_encoding() {
    let mut node = hy2_node(None);
    node.protocol = Protocol::Vmess;
    node.uuid = Some("00000000-0000-0000-0000-000000000001".to_string());
    node.password = None;
    node.network = "tcp".to_string();
    node.extensions = Some(BTreeMap::from([(
        "scy".to_string(),
        serde_json::json!("aes-128-cfb"),
    )]));
    assert!(
        generate_singbox_probe_config(&node, 1080)
            .unwrap_err()
            .contains("unsupported VMess security")
    );

    node.extensions = Some(BTreeMap::from([(
        "packet_encoding".to_string(),
        serde_json::json!("bogus"),
    )]));
    assert!(
        generate_singbox_probe_config(&node, 1080)
            .unwrap_err()
            .contains("unsupported VMess packet_encoding")
    );
}

#[test]
fn rejects_native_invalid_quic_and_vmess_ctr() {
    for protocol in [Protocol::Vless, Protocol::Vmess, Protocol::Trojan] {
        let mut node = hy2_node(None);
        node.protocol = protocol;
        node.uuid = Some("00000000-0000-0000-0000-000000000001".into());
        node.network = "quic".into();
        node.tls = None;
        node.sni = None;
        assert!(
            generate_singbox_probe_config(&node, 1080)
                .unwrap_err()
                .contains("QUIC transport requires TLS")
        );
    }
    let mut node = hy2_node(None);
    node.protocol = Protocol::Vmess;
    node.uuid = Some("00000000-0000-0000-0000-000000000001".into());
    node.network = "tcp".into();
    node.extensions = Some(BTreeMap::from([(
        "scy".into(),
        serde_json::json!("aes-128-ctr"),
    )]));
    assert!(
        generate_singbox_probe_config(&node, 1080)
            .unwrap_err()
            .contains("unsupported VMess security")
    );
}

#[test]
fn rejects_native_unsupported_shadowsocks_inbound_methods() {
    for method in [
        "aes-128-ctr",
        "aes-192-ctr",
        "aes-256-ctr",
        "aes-128-cfb",
        "aes-192-cfb",
        "aes-256-cfb",
        "rc4-md5",
        "chacha20-ietf",
        "xchacha20",
    ] {
        assert!(
            SingboxInbound::shadowsocks("ss-in", "127.0.0.1", 8388, "tcp", method, "secret")
                .unwrap_err()
                .contains("unsupported Shadowsocks inbound method")
        );
    }
}

#[test]
fn generates_httpupgrade_and_rejects_unknown_transport() {
    let mut node = hy2_node(None);
    node.protocol = Protocol::Vless;
    node.uuid = Some("00000000-0000-0000-0000-000000000001".to_string());
    node.password = None;
    node.network = "httpupgrade".to_string();
    node.host = Some("upgrade.example.com".to_string());
    node.path = Some("/upgrade".to_string());

    let config = generate_singbox_probe_config(&node, 1080).unwrap();
    assert_eq!(config.outbounds[0]["transport"]["type"], "httpupgrade");
    assert_eq!(
        config.outbounds[0]["transport"]["host"],
        "upgrade.example.com"
    );
    assert_eq!(config.outbounds[0]["transport"]["path"], "/upgrade");

    node.network = "mkcp".to_string();
    assert!(
        generate_singbox_probe_config(&node, 1080)
            .unwrap_err()
            .contains("unsupported sing-box transport")
    );
}

#[test]
fn rejects_shadowsocks_legacy_cipher_and_accepts_supported_methods() {
    let mut node = hy2_node(None);
    node.protocol = Protocol::Ss;
    node.password = Some("secret".to_string());
    node.tls = None;
    node.sni = None;
    node.network = "tcp".to_string();

    node.method = Some("2022-blake3-aes-128-gcm".to_string());
    node.password = Some("MDEyMzQ1Njc4OWFiY2RlZg==".to_string());
    let config = generate_singbox_probe_config(&node, 1080).unwrap();
    assert_eq!(config.outbounds[0]["method"], "2022-blake3-aes-128-gcm");

    node.password = Some("secret".to_string());
    assert!(
        generate_singbox_probe_config(&node, 1080)
            .unwrap_err()
            .contains("requires a base64 key")
    );

    node.method = Some("chacha20-poly1305".to_string());
    assert!(
        generate_singbox_probe_config(&node, 1080)
            .unwrap_err()
            .contains("unsupported Shadowsocks method")
    );
}

#[test]
fn parses_socks_and_http_tls_requirements() {
    let mut socks = hy2_node(None);
    socks.protocol = Protocol::Socks5;
    socks.password = None;
    socks.tls = None;
    socks.sni = None;
    socks.network = "tcp".to_string();
    socks.username = Some("user".to_string());
    socks.password = Some("pass".to_string());
    let config = generate_singbox_probe_config(&socks, 1080).unwrap();
    assert_eq!(config.outbounds[0]["type"], "socks");
    assert_eq!(config.outbounds[0]["username"], "user");
    assert!(config.outbounds[0].get("tls").is_none());

    let mut partial = socks.clone();
    partial.username = None;
    assert!(
        generate_singbox_probe_config(&partial, 1080)
            .unwrap_err()
            .contains("password requires username")
    );

    let mut https = hy2_node(None);
    https.protocol = Protocol::Http;
    https.password = None;
    https.method = None;
    https.network = "tcp".to_string();
    https.tls = Some("tls".to_string());
    https.sni = Some("proxy.example.com".to_string());
    let config = generate_singbox_probe_config(&https, 1080).unwrap();
    assert_eq!(config.outbounds[0]["type"], "http");
    assert_eq!(
        config.outbounds[0]["tls"]["server_name"],
        "proxy.example.com"
    );
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
