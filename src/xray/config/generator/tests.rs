use super::{
    enable_stats_api, generate_probe_config, generate_probe_config_with_options,
    generate_runtime_config, generate_runtime_config_for_inbounds,
    generate_runtime_config_for_inbounds_with_options,
};
use crate::config::parse_link;
use crate::model::{Node, Protocol};
use crate::xray::config::{
    FragmentOptions, MuxOptions, XrayCompatibilityTarget, XrayDnsConfig, XrayDnsHostValue,
    XrayGenOptions, XrayRouteList, XrayRoutingOptions,
};
use std::collections::BTreeMap;
use std::io::Write;
use std::process::Command;

fn vless_tls_node() -> Node {
    Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("test-uuid".to_string()),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("example.com".to_string()),
        host: None,
        path: None,
        name: Some("test".to_string()),
        extensions: None,
        raw_config: "".to_string(),
    }
}

fn socks_node() -> Node {
    Node {
        protocol: Protocol::Socks5,
        address: "example.com".to_string(),
        port: 1080,
        username: None,
        uuid: None,
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        name: Some("socks".to_string()),
        extensions: None,
        raw_config: "".to_string(),
    }
}

fn assert_transport_selectors(stream: &serde_json::Value, expected: &str) {
    assert_eq!(stream["network"], expected);
    assert!(stream.get("method").is_none());
}

#[test]
fn compatibility_target_selects_transport_key_and_mkcp_schema() {
    let stable =
        parse_link("vless://test-uuid@127.0.0.1:443?type=mkcp&congestion=true&readBufferSize=4")
            .unwrap()
            .unwrap();
    let stable = generate_probe_config(&stable, 1080).unwrap();
    let stable = serde_json::to_value(&stable.outbounds[0]).unwrap();
    assert_eq!(stable["streamSettings"]["network"], "mkcp");
    assert!(stable["streamSettings"].get("method").is_none());
    assert_eq!(stable["streamSettings"]["kcpSettings"]["congestion"], true);
    assert_eq!(stable["streamSettings"]["kcpSettings"]["readBufferSize"], 4);

    let prerelease = parse_link(
        "vless://test-uuid@127.0.0.1:443?type=mkcp&cwndMultiplier=2&maxSendingWindow=2048",
    )
    .unwrap()
    .unwrap();
    let options = XrayGenOptions {
        compatibility: XrayCompatibilityTarget::PrereleaseV26_7_28,
        ..Default::default()
    };
    let prerelease = generate_probe_config_with_options(&prerelease, 1080, &options).unwrap();
    let prerelease = serde_json::to_value(&prerelease.outbounds[0]).unwrap();
    assert_eq!(prerelease["streamSettings"]["method"], "mkcp");
    assert!(prerelease["streamSettings"].get("network").is_none());
    assert_eq!(
        prerelease["streamSettings"]["kcpSettings"]["cwndMultiplier"],
        2
    );
    assert_eq!(
        prerelease["streamSettings"]["kcpSettings"]["maxSendingWindow"],
        2048
    );
}

#[test]
fn stable_mkcp_rejects_legacy_seed_and_header_at_generation() {
    let node = parse_link("vless://test-uuid@127.0.0.1:443?type=mkcp&seed=legacy&headerType=none")
        .unwrap()
        .unwrap();
    let error = generate_probe_config(&node, 1080).unwrap_err();
    assert!(error.contains("parsed by Xray v26.3.27 but rejected during build"));
}

#[test]
fn default_options_leave_config_unchanged() {
    let node = vless_tls_node();
    let baseline = generate_probe_config(&node, 10808).unwrap();
    let tuned =
        generate_probe_config_with_options(&node, 10808, &XrayGenOptions::default()).unwrap();

    let baseline_json = serde_json::to_value(&baseline).unwrap();
    let tuned_json = serde_json::to_value(&tuned).unwrap();
    assert_eq!(baseline_json, tuned_json);
    assert_eq!(tuned.outbounds.len(), 1);
    assert!(tuned_json["outbounds"][0].get("mux").is_none());
}

#[test]
fn generated_runtime_config_satisfies_strict_xray_schema() {
    let config = generate_runtime_config(&vless_tls_node(), 1080, Some(8080)).unwrap();
    let json = serde_json::to_string(&config).unwrap();
    crate::xray::parsing::XrayConfig::from_json_strict(&json).unwrap();
}

#[test]
fn official_share_link_security_extensions_generate_runtime_fields() {
    let node = parse_link("vless://00000000-0000-0000-0000-000000000001@example.com:443?type=ws&security=tls&sni=cdn.example.com&ech=config-list&pcs=certificate-pin&vcn=peer.example&fm=%7B%22type%22%3A%22x%22%7D")
        .unwrap()
        .unwrap();
    let config = generate_probe_config(&node, 1080).unwrap();
    let stream = serde_json::to_value(&config.outbounds[0].stream_settings).unwrap();
    assert_eq!(stream["tlsSettings"]["echConfigList"], "config-list");
    assert_eq!(
        stream["tlsSettings"]["pinnedPeerCertSha256"],
        "certificate-pin"
    );
    assert_eq!(
        stream["tlsSettings"]["verifyPeerCertByName"],
        "peer.example"
    );
    assert_eq!(stream["finalmask"]["type"], "x");

    let reality = parse_link("vless://00000000-0000-0000-0000-000000000001@example.com:443?type=raw&security=reality&sni=cdn.example.com&pbk=public-key&pqv=post-quantum-key")
        .unwrap()
        .unwrap();
    let config = generate_probe_config(&reality, 1080).unwrap();
    let stream = serde_json::to_value(&config.outbounds[0].stream_settings).unwrap();
    assert_eq!(
        stream["realitySettings"]["mldsa65Verify"],
        "post-quantum-key"
    );
}

#[test]
fn malformed_finalmask_and_removed_allow_insecure_fail_explicitly() {
    let malformed = parse_link(
        "vless://00000000-0000-0000-0000-000000000001@example.com:443?type=raw&fm=not-json",
    )
    .unwrap()
    .unwrap();
    let error = generate_probe_config(&malformed, 1080).unwrap_err();
    assert!(error.contains("link parameter \"fm\" must contain valid JSON"));

    let insecure = parse_link("vless://00000000-0000-0000-0000-000000000001@example.com:443?type=raw&security=tls&allowInsecure=1")
        .unwrap()
        .unwrap();
    let options = XrayGenOptions {
        compatibility: XrayCompatibilityTarget::PrereleaseV26_7_28,
        ..Default::default()
    };
    let error = generate_probe_config_with_options(&insecure, 1080, &options).unwrap_err();
    assert!(error.contains("allowInsecure is not supported"));
}

#[test]
fn managed_and_probe_configs_emit_xray_dns() {
    let node = vless_tls_node();
    let mut hosts = BTreeMap::new();
    hosts.insert(
        "full:example.test".to_string(),
        XrayDnsHostValue::One("192.0.2.10".to_string()),
    );
    let options = XrayGenOptions {
        dns: Some(XrayDnsConfig {
            servers: vec!["8.8.8.8".to_string()],
            hosts,
            query_strategy: "UseIPv4".to_string(),
            use_system_hosts: true,
            disable_cache: true,
            disable_fallback: false,
            enable_parallel_query: true,
        }),
        ..Default::default()
    };

    let runtime = generate_runtime_config_for_inbounds_with_options(
        &node,
        Some(("127.0.0.1", 1080, true)),
        None,
        &options,
    )
    .unwrap();
    let runtime_json = serde_json::to_value(&runtime).unwrap();
    assert_eq!(runtime_json["dns"]["queryStrategy"], "UseIPv4");
    assert_eq!(runtime_json["dns"]["useSystemHosts"], true);
    assert_eq!(runtime_json["dns"]["disableCache"], true);
    assert_eq!(
        runtime_json["dns"]["hosts"]["full:example.test"],
        "192.0.2.10"
    );

    serde_json::from_value::<crate::xray::parsing::core::DnsObject>(runtime_json["dns"].clone())
        .expect("generated Xray DNS object should satisfy the parser schema");

    let probe = generate_probe_config_with_options(&node, 1080, &options).unwrap();
    let probe_json = serde_json::to_value(probe).unwrap();
    assert_eq!(probe_json["dns"]["queryStrategy"], "UseIPv4");
    assert_eq!(probe_json["dns"]["useSystemHosts"], true);
}

#[test]
fn native_xray_validator_accepts_generated_dns_config() {
    if Command::new("xray").arg("version").output().is_err() {
        return;
    }

    let node = vless_tls_node();
    let options = XrayGenOptions {
        dns: Some(XrayDnsConfig {
            servers: vec!["8.8.8.8".to_string()],
            hosts: BTreeMap::new(),
            query_strategy: "UseIPv4".to_string(),
            use_system_hosts: true,
            disable_cache: false,
            disable_fallback: false,
            enable_parallel_query: true,
        }),
        ..Default::default()
    };
    let config = generate_probe_config_with_options(&node, 1080, &options).unwrap();
    let mut file = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    file.write_all(serde_json::to_string(&config).unwrap().as_bytes())
        .unwrap();
    file.flush().unwrap();

    let output = Command::new("xray")
        .args(["run", "-test", "-c"])
        .arg(file.path())
        .output()
        .expect("xray should start");
    assert!(
        output.status.success(),
        "xray rejected generated DNS config: stderr={}, stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn native_xray_validators_accept_targeted_mkcp_configs() {
    let stable_binary = std::env::var_os("XRAT_STABLE_XRAY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("xray"));
    if Command::new(&stable_binary)
        .arg("version")
        .output()
        .is_err()
    {
        return;
    }
    let stable_node = parse_link(
        "vless://00000000-0000-0000-0000-000000000001@127.0.0.1:443?type=mkcp&congestion=true&readBufferSize=4",
    )
    .unwrap()
    .unwrap();
    let stable = generate_probe_config(&stable_node, 1080).unwrap();
    assert_native_xray_config(&stable_binary, &stable);

    let Some(prerelease_binary) = std::env::var_os("XRAT_PRERELEASE_XRAY") else {
        return;
    };
    let prerelease_node = parse_link(
        "vless://00000000-0000-0000-0000-000000000001@127.0.0.1:443?type=mkcp&cwndMultiplier=2&maxSendingWindow=2048",
    )
    .unwrap()
    .unwrap();
    let options = XrayGenOptions {
        compatibility: XrayCompatibilityTarget::PrereleaseV26_7_28,
        ..Default::default()
    };
    let prerelease = generate_probe_config_with_options(&prerelease_node, 1080, &options).unwrap();
    assert_native_xray_config(std::path::Path::new(&prerelease_binary), &prerelease);
}

fn assert_native_xray_config(binary: &std::path::Path, config: &crate::xray::config::XrayConfig) {
    let mut file = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    file.write_all(serde_json::to_string(config).unwrap().as_bytes())
        .unwrap();
    file.flush().unwrap();
    let output = Command::new(binary)
        .args(["run", "-test", "-c"])
        .arg(file.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{} rejected generated config: stderr={}, stdout={}",
        binary.display(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn mux_enabled_emits_camelcase_keys_on_proxy_only() {
    let node = vless_tls_node();
    let options = XrayGenOptions {
        mux: Some(MuxOptions {
            concurrency: 4,
            xudp_concurrency: 16,
            xudp_proxy_udp443: "skip".to_string(),
        }),
        ..Default::default()
    };
    let config =
        generate_runtime_config_for_inbounds_with_options(&node, None, None, &options).unwrap();

    assert_eq!(config.outbounds.len(), 1);
    let json = serde_json::to_value(&config.outbounds[0]).unwrap();
    let mux = json.get("mux").expect("mux should be present");
    assert_eq!(mux["enabled"], true);
    assert_eq!(mux["concurrency"], 4);
    assert_eq!(mux["xudpConcurrency"], 16);
    assert_eq!(mux["xudpProxyUDP443"], "skip");
}

#[test]
fn fragment_enabled_adds_freedom_outbound_and_dialer_proxy() {
    let node = vless_tls_node();
    let options = XrayGenOptions {
        fragment: Some(FragmentOptions {
            packets: "tlshello".to_string(),
            length: "100-200".to_string(),
            interval: "10-20".to_string(),
        }),
        ..Default::default()
    };
    let config = generate_probe_config_with_options(&node, 10808, &options).unwrap();

    assert_eq!(config.outbounds.len(), 2);
    let proxy = &config.outbounds[0];
    assert_eq!(proxy.tag, "proxy");
    // Transport settings are preserved on the proxy outbound.
    let stream = proxy.stream_settings.as_ref().unwrap();
    assert!(stream.tls_settings.is_some());
    assert_eq!(
        stream.sockopt.as_ref().unwrap().dialer_proxy.as_deref(),
        Some("fragment")
    );

    let fragment = &config.outbounds[1];
    assert_eq!(fragment.tag, "fragment");
    assert_eq!(fragment.protocol, "freedom");
    assert_eq!(fragment.settings["fragment"]["packets"], "tlshello");
    assert_eq!(fragment.settings["fragment"]["length"], "100-200");
    assert_eq!(fragment.settings["fragment"]["interval"], "10-20");
}

#[test]
fn interface_only_binds_proxy_outbound() {
    let node = vless_tls_node();
    let options = XrayGenOptions {
        interface: Some("eth0".to_string()),
        mark: Some(255),
        ..Default::default()
    };
    let config = generate_probe_config_with_options(&node, 10808, &options).unwrap();

    assert_eq!(config.outbounds.len(), 1);
    let sockopt = config.outbounds[0]
        .stream_settings
        .as_ref()
        .unwrap()
        .sockopt
        .as_ref()
        .unwrap();
    assert_eq!(sockopt.interface.as_deref(), Some("eth0"));
    assert_eq!(sockopt.mark, Some(255));
}

#[test]
fn interface_with_fragment_binds_fragment_outbound() {
    let node = vless_tls_node();
    let options = XrayGenOptions {
        fragment: Some(FragmentOptions {
            packets: "tlshello".to_string(),
            length: "100-200".to_string(),
            interval: "10-20".to_string(),
        }),
        interface: Some("eth0".to_string()),
        ..Default::default()
    };
    let config = generate_probe_config_with_options(&node, 10808, &options).unwrap();

    // Interface binding belongs on the egress (fragment) outbound, not proxy.
    let proxy_sockopt = config.outbounds[0]
        .stream_settings
        .as_ref()
        .unwrap()
        .sockopt
        .as_ref()
        .unwrap();
    assert!(proxy_sockopt.interface.is_none());
    assert_eq!(proxy_sockopt.dialer_proxy.as_deref(), Some("fragment"));

    let fragment_sockopt = config.outbounds[1]
        .stream_settings
        .as_ref()
        .unwrap()
        .sockopt
        .as_ref()
        .unwrap();
    assert_eq!(fragment_sockopt.interface.as_deref(), Some("eth0"));
}

#[test]
fn managed_runtime_emits_ordered_routing_rules_and_outbounds() {
    let node = vless_tls_node();
    let options = XrayGenOptions {
        interface: Some("eth0".to_string()),
        mark: Some(255),
        routing: Some(XrayRoutingOptions {
            domain_strategy: "IPIfNonMatch".to_string(),
            direct: XrayRouteList {
                domain: vec!["domain:direct.example".to_string()],
                ip: vec!["192.168.0.0/16".to_string()],
                geosite: vec!["private".to_string()],
                geoip: vec!["private".to_string()],
            },
            block: XrayRouteList {
                domain: vec!["domain:ads.example".to_string()],
                ip: vec!["203.0.113.0/24".to_string()],
                geosite: vec!["category-ads-all".to_string()],
                geoip: vec!["cn".to_string()],
            },
        }),
        ..Default::default()
    };

    let config =
        generate_runtime_config_for_inbounds_with_options(&node, None, None, &options).unwrap();
    let value = serde_json::to_value(&config).unwrap();

    assert_eq!(
        value["outbounds"]
            .as_array()
            .unwrap()
            .iter()
            .map(|outbound| outbound["tag"].as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["proxy", "direct", "block"]
    );
    assert_eq!(value["outbounds"][1]["protocol"], "freedom");
    assert_eq!(value["outbounds"][2]["protocol"], "blackhole");
    assert_eq!(
        value["outbounds"][1]["streamSettings"]["sockopt"]["interface"],
        "eth0"
    );
    assert_eq!(
        value["outbounds"][1]["streamSettings"]["sockopt"]["mark"],
        255
    );
    assert_eq!(value["routing"]["domainStrategy"], "IPIfNonMatch");

    let rules = value["routing"]["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 4);
    assert_eq!(rules[0]["outboundTag"], "direct");
    assert_eq!(
        rules[0]["domain"],
        serde_json::json!(["domain:direct.example", "geosite:private"])
    );
    assert!(rules[0].get("ip").is_none());
    assert_eq!(rules[1]["outboundTag"], "direct");
    assert_eq!(
        rules[1]["ip"],
        serde_json::json!(["192.168.0.0/16", "geoip:private"])
    );
    assert_eq!(rules[2]["outboundTag"], "block");
    assert_eq!(
        rules[2]["domain"],
        serde_json::json!(["domain:ads.example", "geosite:category-ads-all"])
    );
    assert_eq!(rules[3]["outboundTag"], "block");
    assert_eq!(
        rules[3]["ip"],
        serde_json::json!(["203.0.113.0/24", "geoip:cn"])
    );
}

#[test]
fn probes_ignore_managed_runtime_routing() {
    let node = vless_tls_node();
    let options = XrayGenOptions {
        routing: Some(XrayRoutingOptions {
            domain_strategy: "AsIs".to_string(),
            direct: XrayRouteList {
                domain: vec!["domain:direct.example".to_string()],
                ..Default::default()
            },
            block: XrayRouteList::default(),
        }),
        ..Default::default()
    };

    let config = generate_probe_config_with_options(&node, 10808, &options).unwrap();

    assert!(config.routing.is_none());
    assert_eq!(config.outbounds.len(), 1);
}

#[test]
fn stats_api_rule_precedes_user_routing_rules() {
    let node = vless_tls_node();
    let options = XrayGenOptions {
        routing: Some(XrayRoutingOptions {
            domain_strategy: "AsIs".to_string(),
            direct: XrayRouteList {
                domain: vec!["domain:direct.example".to_string()],
                ..Default::default()
            },
            block: XrayRouteList::default(),
        }),
        ..Default::default()
    };
    let mut config =
        generate_runtime_config_for_inbounds_with_options(&node, None, None, &options).unwrap();

    enable_stats_api(&mut config, "127.0.0.1", 10085);

    let value = serde_json::to_value(config).unwrap();
    assert_eq!(value["routing"]["rules"][0]["outboundTag"], "api");
    assert_eq!(value["routing"]["rules"][1]["outboundTag"], "direct");
}

#[test]
fn socks_upstream_gets_minimal_stream_settings_for_sockopt() {
    let node = socks_node();
    let options = XrayGenOptions {
        interface: Some("eth0".to_string()),
        ..Default::default()
    };
    let config = generate_probe_config_with_options(&node, 10808, &options).unwrap();

    let stream = config.outbounds[0].stream_settings.as_ref().unwrap();
    assert_eq!(stream.network, "raw");
    assert_eq!(
        stream.sockopt.as_ref().unwrap().interface.as_deref(),
        Some("eth0")
    );
}

#[test]
fn test_generate_vless_probe_config() {
    let node = Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("test-uuid".to_string()),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("example.com".to_string()),
        host: None,
        path: None,
        name: Some("test".to_string()),
        extensions: None,
        raw_config: "".to_string(),
    };

    let config = generate_probe_config(&node, 10808).unwrap();
    assert_eq!(config.inbounds.len(), 1);
    assert_eq!(config.inbounds[0].port, 10808);
    assert_eq!(config.outbounds.len(), 1);
    assert_eq!(config.outbounds[0].protocol, "vless");
}

#[test]
fn enable_stats_api_adds_api_inbound_and_objects() {
    let node = Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("test-uuid".to_string()),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("example.com".to_string()),
        host: None,
        path: None,
        name: Some("test".to_string()),
        extensions: None,
        raw_config: "".to_string(),
    };

    let mut config =
        generate_runtime_config_for_inbounds(&node, Some(("127.0.0.1", 18200, true)), None)
            .unwrap();
    assert!(config.api.is_none());
    enable_stats_api(&mut config, "127.0.0.1", 10085);

    let api = config.inbounds.iter().find(|inbound| inbound.tag == "api");
    assert!(api.is_some(), "api dokodemo inbound should be present");
    assert_eq!(api.unwrap().port, 10085);
    assert_eq!(api.unwrap().protocol, "dokodemo-door");
    assert!(config.api.is_some());
    assert!(config.stats.is_some());

    let value = serde_json::to_value(&config).unwrap();
    assert_eq!(value["api"]["services"][0], "StatsService");
    assert_eq!(value["policy"]["system"]["statsInboundUplink"], true);
    assert_eq!(value["routing"]["rules"][0]["outboundTag"], "api");
}

#[test]
fn test_generate_vmess_ws_config() {
    let node = Node {
        protocol: Protocol::Vmess,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("test-uuid".to_string()),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: None,
        host: Some("example.com".to_string()),
        path: Some("/path".to_string()),
        name: Some("test".to_string()),
        extensions: None,
        raw_config: "".to_string(),
    };

    let config = generate_runtime_config(&node, 1080, Some(8080)).unwrap();
    assert_eq!(config.inbounds.len(), 2);
    assert_eq!(config.outbounds[0].protocol, "vmess");

    let stream = config.outbounds[0].stream_settings.as_ref().unwrap();
    assert_eq!(stream.network, "websocket");
    assert!(stream.ws_settings.is_some());
}

#[test]
fn serializes_stream_settings_with_xray_camel_case_key() {
    let node = Node {
        protocol: Protocol::Vless,
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: Some("test-uuid".to_string()),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("example.com".to_string()),
        host: None,
        path: None,
        name: Some("test".to_string()),
        extensions: None,
        raw_config: "".to_string(),
    };

    let config = generate_probe_config(&node, 10808).unwrap();
    let json = serde_json::to_value(&config.outbounds[0]).unwrap();
    assert!(
        json.get("streamSettings").is_some(),
        "xray requires the camelCase streamSettings key, got: {json}"
    );
    assert!(json.get("stream_settings").is_none());
}

#[test]
fn generates_xhttp_stream_with_tls_fingerprint_and_alpn() {
    let mut extensions = std::collections::BTreeMap::new();
    extensions.insert("fp".to_string(), serde_json::json!("chrome"));
    extensions.insert("alpn".to_string(), serde_json::json!("h2"));
    extensions.insert("mode".to_string(), serde_json::json!("auto"));

    let node = Node {
        protocol: Protocol::Vless,
        address: "ip2.example.com".to_string(),
        port: 2087,
        username: None,
        uuid: Some("test-uuid".to_string()),
        password: None,
        method: None,
        network: "xhttp".to_string(),
        tls: Some("tls".to_string()),
        sni: Some("cdn.example.com".to_string()),
        host: Some("cdn.example.com".to_string()),
        path: None,
        name: Some("test".to_string()),
        extensions: Some(extensions),
        raw_config: "".to_string(),
    };

    let config = generate_probe_config(&node, 10808).unwrap();
    let stream = config.outbounds[0].stream_settings.as_ref().unwrap();

    assert_eq!(stream.network, "xhttp");
    let json = serde_json::to_value(&config.outbounds[0]).unwrap();
    assert_transport_selectors(&json["streamSettings"], "xhttp");
    let xhttp = stream.xhttp_settings.as_ref().unwrap();
    assert_eq!(xhttp.host.as_deref(), Some("cdn.example.com"));
    assert_eq!(xhttp.path, "/");
    assert_eq!(xhttp.mode.as_deref(), Some("auto"));

    let tls = stream.tls_settings.as_ref().unwrap();
    assert_eq!(tls.server_name, "cdn.example.com");
    assert_eq!(tls.fingerprint.as_deref(), Some("chrome"));
    assert_eq!(tls.alpn.as_deref(), Some(&["h2".to_string()][..]));
}

#[test]
fn vless_xhttp_link_merges_alias_extra_and_canonical_parameters() {
    let link = concat!(
        "vless://test-uuid@example.com:443?type=xhttp&security=tls&mode=auto",
        "&x_padding%20bytes=1-2&x_padding_bytes=1-2",
        "&extra=%7B%22futureOption%22%3Atrue%2C%22xPaddingBytes%22%3A%223-4%22%7D",
        "&xPaddingBytes=5-6&noSSEHeader=true#XHTTP"
    );
    let node = parse_link(link).unwrap().unwrap();
    let config = generate_probe_config(&node, 10808).unwrap();
    let json = serde_json::to_value(config).unwrap();
    let extra = &json["outbounds"][0]["streamSettings"]["xhttpSettings"]["extra"];

    assert_eq!(extra["xPaddingBytes"], "5-6");
    assert_eq!(extra["noSSEHeader"], true);
    assert_eq!(extra["futureOption"], true);
}

#[test]
fn xhttp_padding_aliases_are_supported_individually() {
    for parameter in ["x_padding%20bytes", "x_padding_bytes", "xPaddingBytes"] {
        let link =
            format!("vless://test-uuid@example.com:443?type=xhttp&{parameter}=100-200#XHTTP");
        let node = parse_link(&link).unwrap().unwrap();
        let config = generate_probe_config(&node, 10808).unwrap();
        let json = serde_json::to_value(config).unwrap();
        assert_eq!(
            json["outbounds"][0]["streamSettings"]["xhttpSettings"]["extra"]["xPaddingBytes"],
            "100-200"
        );
    }
}

#[test]
fn xhttp_ignores_neutral_legacy_header_type() {
    let baseline = parse_link("vless://test-uuid@example.com:443?type=xhttp#XHTTP")
        .unwrap()
        .unwrap();
    let baseline = serde_json::to_value(generate_probe_config(&baseline, 10808).unwrap()).unwrap();

    for header_type in ["", "none"] {
        let link =
            format!("vless://test-uuid@example.com:443?type=xhttp&headerType={header_type}#XHTTP");
        let node = parse_link(&link).unwrap().unwrap();
        let config = serde_json::to_value(generate_probe_config(&node, 10808).unwrap()).unwrap();

        assert_eq!(config, baseline);
    }
}

#[test]
fn xhttp_rejects_malformed_repeated_conflicting_and_unknown_parameters() {
    for (query, expected) in [
        ("extra=not-json", "valid JSON"),
        ("extra=%5B1%2C2%5D", "JSON object"),
        ("noSSEHeader=maybe", "must be true"),
        (
            "xPaddingBytes=1&xPaddingBytes=2",
            "duplicate query parameter",
        ),
        (
            "x_padding_bytes=1-2&x_padding%20bytes=3-4",
            "conflicting link parameters",
        ),
        ("headerType=http", "headerType"),
        ("futureFlatOption=on", "JSON `extra` parameter"),
    ] {
        let link = format!("vless://test-uuid@example.com:443?type=xhttp&{query}#XHTTP");
        let error = match parse_link(&link) {
            Ok(Some(node)) => generate_probe_config(&node, 10808).unwrap_err(),
            Err(error) => error.to_string(),
            Ok(None) => panic!("link was not parsed"),
        };
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}

#[test]
fn xhttp_rejects_conflicting_structural_fields_inside_extra() {
    let link = concat!(
        "vless://test-uuid@example.com:443?type=xhttp&path=%2Fouter",
        "&extra=%7B%22path%22%3A%22%2Finner%22%7D#XHTTP"
    );
    let node = parse_link(link).unwrap().unwrap();
    assert!(
        generate_probe_config(&node, 10808)
            .unwrap_err()
            .contains("extra.path")
    );
}

#[test]
fn rejects_parameters_used_by_the_wrong_transport() {
    let node =
        parse_link("vless://test-uuid@example.com:443?type=ws&security=tls&mtu=1350#WebSocket")
            .unwrap()
            .unwrap();
    let error = generate_probe_config(&node, 10808).unwrap_err();
    assert!(error.contains("mtu"), "unexpected error: {error}");
}

#[test]
fn native_xray_validator_accepts_generated_xhttp_config() {
    if Command::new("xray").arg("version").output().is_err() {
        return;
    }

    let link = concat!(
        "vless://test-uuid@example.com:443?type=xhttp&security=tls&mode=auto",
        "&xPaddingBytes=100-200&noSSEHeader=true#XHTTP"
    );
    let node = parse_link(link).unwrap().unwrap();
    let config = generate_probe_config(&node, 10808).unwrap();
    let json = serde_json::to_value(&config).unwrap();
    assert_transport_selectors(&json["outbounds"][0]["streamSettings"], "xhttp");
    let mut file = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
    file.write_all(serde_json::to_string(&config).unwrap().as_bytes())
        .unwrap();
    file.flush().unwrap();

    let output = Command::new("xray")
        .args(["run", "-test", "-c"])
        .arg(file.path())
        .output()
        .expect("xray should start");
    assert!(
        output.status.success(),
        "xray rejected generated XHTTP config: stderr={}, stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn generates_reality_stream_with_settings_and_flow() {
    let mut extensions = std::collections::BTreeMap::new();
    extensions.insert("pbk".to_string(), serde_json::json!("test-public-key"));
    extensions.insert("sid".to_string(), serde_json::json!("0123abcd"));
    extensions.insert("spx".to_string(), serde_json::json!("/"));
    extensions.insert("flow".to_string(), serde_json::json!("xtls-rprx-vision"));

    let node = Node {
        protocol: Protocol::Vless,
        address: "1.2.3.4".to_string(),
        port: 443,
        username: None,
        uuid: Some("test-uuid".to_string()),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("reality".to_string()),
        sni: Some("www.example.com".to_string()),
        host: None,
        path: None,
        name: Some("reality-node".to_string()),
        extensions: Some(extensions),
        raw_config: "".to_string(),
    };

    let config = generate_probe_config(&node, 10808).unwrap();
    let stream = config.outbounds[0].stream_settings.as_ref().unwrap();

    assert_eq!(stream.security.as_deref(), Some("reality"));
    let reality = stream.reality_settings.as_ref().unwrap();
    assert_eq!(reality.server_name, "www.example.com");
    assert_eq!(reality.public_key, "test-public-key");
    assert_eq!(reality.short_id.as_deref(), Some("0123abcd"));
    assert_eq!(reality.spider_x.as_deref(), Some("/"));
    assert_eq!(reality.fingerprint.as_deref(), Some("chrome"));

    let settings = &config.outbounds[0].settings;
    assert_eq!(settings["vnext"][0]["users"][0]["flow"], "xtls-rprx-vision");

    let json = serde_json::to_value(&config.outbounds[0]).unwrap();
    assert!(
        json["streamSettings"].get("realitySettings").is_some(),
        "xray requires camelCase realitySettings, got: {json}"
    );
}

#[test]
fn generates_http_only_runtime_config() {
    let node = Node {
        protocol: Protocol::Http,
        address: "example.com".to_string(),
        port: 8080,
        username: Some("user".to_string()),
        uuid: None,
        password: Some("pass".to_string()),
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        name: Some("http".to_string()),
        extensions: None,
        raw_config: "".to_string(),
    };

    let config =
        generate_runtime_config_for_inbounds(&node, None, Some(("127.0.0.1", 18080))).unwrap();

    assert_eq!(config.inbounds.len(), 1);
    assert_eq!(config.inbounds[0].protocol, "http");
    assert_eq!(config.inbounds[0].port, 18080);
}

#[test]
fn emits_current_reality_and_transport_field_names() {
    let mut node = vless_tls_node();
    node.tls = Some("reality".to_string());
    node.extensions = Some(std::collections::BTreeMap::from([
        ("pbk".to_string(), serde_json::json!("key")),
        ("sid".to_string(), serde_json::json!("abcd")),
    ]));

    let config = generate_probe_config(&node, 10808).unwrap();
    let json = serde_json::to_value(&config.outbounds[0]).unwrap();
    assert_transport_selectors(&json["streamSettings"], "raw");
    assert_eq!(json["streamSettings"]["realitySettings"]["password"], "key");
    assert!(
        json["streamSettings"]["realitySettings"]
            .get("publicKey")
            .is_none()
    );
}

#[test]
fn generates_grpc_mkcp_and_httpupgrade_settings() {
    let mut grpc = vless_tls_node();
    grpc.network = "grpc".to_string();
    grpc.extensions = Some(std::collections::BTreeMap::from([
        ("serviceName".to_string(), serde_json::json!("xrat")),
        ("multiMode".to_string(), serde_json::json!(true)),
    ]));
    let grpc_json = serde_json::to_value(generate_probe_config(&grpc, 10808).unwrap()).unwrap();
    assert_transport_selectors(&grpc_json["outbounds"][0]["streamSettings"], "grpc");
    assert_eq!(
        grpc_json["outbounds"][0]["streamSettings"]["grpcSettings"]["serviceName"],
        "xrat"
    );
    assert_eq!(
        grpc_json["outbounds"][0]["streamSettings"]["grpcSettings"]["multiMode"],
        true
    );

    let mut mkcp = vless_tls_node();
    mkcp.network = "kcp".to_string();
    mkcp.tls = None;
    mkcp.extensions = Some(std::collections::BTreeMap::from([
        ("mtu".to_string(), serde_json::json!(1350)),
        ("congestion".to_string(), serde_json::json!(true)),
    ]));
    let mkcp_json = serde_json::to_value(generate_probe_config(&mkcp, 10809).unwrap()).unwrap();
    assert_transport_selectors(&mkcp_json["outbounds"][0]["streamSettings"], "mkcp");
    assert_eq!(
        mkcp_json["outbounds"][0]["streamSettings"]["kcpSettings"]["mtu"],
        1350
    );

    let mut upgrade = vless_tls_node();
    upgrade.network = "httpupgrade".to_string();
    upgrade.path = Some("/upgrade".to_string());
    upgrade.host = Some("cdn.example.com".to_string());
    let upgrade_json =
        serde_json::to_value(generate_probe_config(&upgrade, 10810).unwrap()).unwrap();
    assert_transport_selectors(
        &upgrade_json["outbounds"][0]["streamSettings"],
        "httpupgrade",
    );
    assert_eq!(
        upgrade_json["outbounds"][0]["streamSettings"]["httpupgradeSettings"]["path"],
        "/upgrade"
    );
}

#[test]
fn rejects_removed_transports_and_unknown_wire_parameters() {
    let mut node = vless_tls_node();
    node.network = "h2".to_string();
    assert!(
        generate_probe_config(&node, 10808)
            .unwrap_err()
            .contains("removed Xray transport")
    );

    node.network = "ws".to_string();
    node.extensions = Some(std::collections::BTreeMap::from([(
        "futureWireOption".to_string(),
        serde_json::json!("on"),
    )]));
    assert!(
        generate_probe_config(&node, 10808)
            .unwrap_err()
            .contains("futureWireOption")
    );
}
