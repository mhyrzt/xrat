use serde_json::json;
use std::collections::BTreeMap;
use xrat_engines::singbox::*;
use xrat_model::{Node, Protocol};

fn node(protocol: Protocol) -> Node {
    Node {
        protocol,
        address: "192.0.2.1".into(),
        port: 443,
        username: Some("fixture-user".into()),
        uuid: Some("00000000-0000-0000-0000-000000000001".into()),
        password: Some("fixture-password".into()),
        method: Some("aes-128-gcm".into()),
        network: "tcp".into(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        name: None,
        extensions: None,
        raw_config: String::new(),
    }
}

fn extend(node: &mut Node, key: &str, value: &str) {
    node.extensions
        .get_or_insert_with(BTreeMap::new)
        .insert(key.into(), json!(value));
}

fn nodes() -> Vec<(String, Node)> {
    let mut nodes = Vec::new();
    for protocol in [Protocol::Vless, Protocol::Vmess, Protocol::Trojan] {
        for transport in ["tcp", "ws", "grpc", "httpupgrade", "h2", "quic"] {
            for security in ["none", "tls", "reality"] {
                if transport == "quic" && security == "none" {
                    continue;
                }
                let mut node = node(protocol.clone());
                node.network = transport.into();
                if security != "none" {
                    node.tls = Some(security.into());
                    node.sni = Some("fixture.example".into());
                    extend(&mut node, "alpn", "h2,http/1.1");
                    extend(&mut node, "insecure", "1");
                    extend(&mut node, "fp", "chrome");
                    extend(&mut node, "cs", "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256");
                }
                if security == "reality" {
                    extend(
                        &mut node,
                        "pbk",
                        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                    );
                    extend(&mut node, "sid", "0123456789abcdef");
                }
                if matches!(transport, "ws" | "httpupgrade" | "h2") {
                    node.host = Some("front.example".into());
                    node.path = Some("/fixture".into());
                } else if transport == "grpc" {
                    extend(&mut node, "serviceName", "FixtureService");
                }
                nodes.push((format!("{protocol}-{transport}-{security}"), node));
            }
        }
    }
    for encoding in ["packetaddr", "xudp"] {
        for protocol in [Protocol::Vless, Protocol::Vmess] {
            let mut node = node(protocol.clone());
            extend(&mut node, "packet_encoding", encoding);
            nodes.push((format!("{protocol}-{encoding}"), node));
        }
    }
    for security in ["auto", "none", "zero", "aes-128-gcm", "chacha20-poly1305"] {
        let mut node = node(Protocol::Vmess);
        extend(&mut node, "scy", security);
        extend(&mut node, "aid", "0");
        nodes.push((format!("vmess-security-{security}"), node));
    }
    let mut vision = node(Protocol::Vless);
    vision.tls = Some("tls".into());
    extend(&mut vision, "flow", "xtls-rprx-vision");
    nodes.push(("vless-vision".into(), vision));
    for method in [
        "2022-blake3-aes-128-gcm",
        "2022-blake3-aes-256-gcm",
        "2022-blake3-chacha20-poly1305",
        "none",
        "aes-128-gcm",
        "aes-192-gcm",
        "aes-256-gcm",
        "chacha20-ietf-poly1305",
        "xchacha20-ietf-poly1305",
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
        let mut node = node(Protocol::Ss);
        node.method = Some(method.into());
        node.password = Some(password(method).into());
        nodes.push((format!("ss-{method}"), node));
    }
    for protocol in [Protocol::Http, Protocol::Socks5] {
        for auth in [false, true] {
            let mut node = node(protocol.clone());
            node.password = auth.then(|| "fixture-password".into());
            node.username = auth.then(|| "fixture-user".into());
            nodes.push((format!("{protocol}-auth-{auth}"), node.clone()));
            if protocol == Protocol::Http {
                node.tls = Some("tls".into());
                nodes.push((format!("https-auth-{auth}"), node));
            }
        }
    }
    for obfs in [false, true] {
        let mut node = node(Protocol::Hy2);
        node.network = "udp".into();
        node.raw_config = "hy2://fixture-password@192.0.2.1:443".into();
        node.tls = Some("tls".into());
        if obfs {
            extend(&mut node, "obfs", "salamander");
            extend(&mut node, "obfs-password", "fixture-obfs");
            extend(&mut node, "upmbps", "20");
            extend(&mut node, "downmbps", "80");
            extend(&mut node, "alpn", "h3");
            extend(&mut node, "insecure", "1");
        }
        nodes.push((format!("hy2-obfs-{obfs}"), node));
    }
    nodes
}

fn password(method: &str) -> &'static str {
    match method {
        "2022-blake3-aes-128-gcm" => "MDEyMzQ1Njc4OWFiY2RlZg==",
        "2022-blake3-aes-256-gcm" | "2022-blake3-chacha20-poly1305" => {
            "MDEyMzQ1Njc4OWFiY2RlZjAxMjM0NTY3ODlhYmNkZWY="
        }
        _ => "fixture-password",
    }
}

fn inbounds(mask: u8) -> Vec<SingboxInbound> {
    let mut inbounds = Vec::new();
    if mask & 1 != 0 {
        inbounds.push(SingboxInbound::socks(
            "socks-in",
            "127.0.0.1",
            1080,
            Some(vec![SingboxInboundUser {
                username: "fixture-user".into(),
                password: "fixture-password".into(),
            }]),
        ));
    }
    if mask & 2 != 0 {
        inbounds.push(SingboxInbound::http("http-in", "127.0.0.1", 8080));
    }
    if mask & 4 != 0 {
        inbounds.push(
            SingboxInbound::shadowsocks(
                "ss-in",
                "127.0.0.1",
                8388,
                "udp",
                "aes-128-gcm",
                "fixture-password",
            )
            .unwrap(),
        );
    }
    if mask & 8 != 0 {
        inbounds.push(
            SingboxInbound::tun(SingboxTunOptions {
                tag: "tun-in".into(),
                interface_name: "xrat-fixture".into(),
                address: vec!["172.19.0.1/30".into(), "fdfe:dcba:9876::1/126".into()],
                mtu: 1500,
                stack: "system".into(),
                auto_route: true,
                strict_route: true,
                route_exclude_address: vec!["192.168.0.0/16".into()],
            })
            .unwrap(),
        );
    }
    inbounds
}

pub fn matrix() -> Vec<(String, SingboxConfig)> {
    let mut fixtures = Vec::new();
    let routing = SingboxRoutingOptions {
        direct: SingboxRouteList {
            domain: vec![
                "full:exact.example".into(),
                "domain:suffix.example".into(),
                "keyword:safe".into(),
                "regexp:^safe\\.example$".into(),
            ],
            ip: vec!["192.168.0.0/16".into()],
            geosite: vec!["private".into()],
            ..Default::default()
        },
        block: SingboxRouteList {
            domain: vec!["domain:ads.example".into()],
            ip: vec!["203.0.113.0/24".into()],
            geoip: vec!["cn".into()],
            ..Default::default()
        },
    };
    for (name, node) in nodes() {
        if node.protocol == Protocol::Ss
            && matches!(
                node.method.as_deref(),
                Some(
                    "2022-blake3-aes-128-gcm"
                        | "2022-blake3-aes-256-gcm"
                        | "2022-blake3-chacha20-poly1305"
                        | "none"
                        | "aes-128-gcm"
                        | "aes-192-gcm"
                        | "aes-256-gcm"
                        | "chacha20-ietf-poly1305"
                        | "xchacha20-ietf-poly1305"
                )
            )
        {
            let method = node.method.as_deref().unwrap();
            for network in ["tcp", "udp"] {
                let inbound = SingboxInbound::shadowsocks(
                    "ss-in",
                    "127.0.0.1",
                    8388,
                    network,
                    method,
                    password(method),
                )
                .unwrap();
                let config =
                    generate_singbox_runtime_config(&node, vec![inbound], None, None).unwrap();
                fixtures.push((format!("inbound-ss-{method}-{network}"), config));
            }
        }
        let probe = generate_singbox_probe_config(&node, 1080)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        fixtures.push((format!("probe-{name}"), probe));
        let mut runtime = generate_singbox_runtime_config(
            &node,
            inbounds(15),
            Some(SingboxClashApi {
                external_controller: "127.0.0.1:9090".into(),
                secret: Some("fixture-secret".into()),
            }),
            Some(&routing),
        )
        .unwrap_or_else(|error| panic!("{name}: {error}"));
        runtime.enable_tun_route();
        runtime.enable_cache_file("fixture-cache.db".into());
        fixtures.push((format!("runtime-{name}"), runtime));
    }
    for mask in 0..16 {
        let mut config =
            generate_singbox_runtime_config(&node(Protocol::Socks5), inbounds(mask), None, None)
                .unwrap();
        if mask & 8 != 0 {
            config.enable_tun_route();
        }
        fixtures.push((format!("inbounds-{mask:04b}"), config));
    }
    for stack in ["system", "gvisor", "mixed"] {
        let mut inbound = inbounds(8);
        if let SingboxInbound::Tun { stack: value, .. } = &mut inbound[0] {
            *value = stack.into();
        }
        let mut config =
            generate_singbox_runtime_config(&node(Protocol::Socks5), inbound, None, None).unwrap();
        config.enable_tun_route();
        fixtures.push((format!("tun-stack-{stack}"), config));
    }
    fixtures
}
