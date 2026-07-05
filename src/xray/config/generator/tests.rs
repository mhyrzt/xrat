use super::{
    enable_stats_api, generate_probe_config, generate_probe_config_with_options,
    generate_runtime_config, generate_runtime_config_for_inbounds,
    generate_runtime_config_for_inbounds_with_options,
};
use crate::model::{Node, Protocol};
use crate::xray::config::{FragmentOptions, MuxOptions, XrayGenOptions};

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
fn socks_upstream_gets_minimal_stream_settings_for_sockopt() {
    let node = socks_node();
    let options = XrayGenOptions {
        interface: Some("eth0".to_string()),
        ..Default::default()
    };
    let config = generate_probe_config_with_options(&node, 10808, &options).unwrap();

    let stream = config.outbounds[0].stream_settings.as_ref().unwrap();
    assert_eq!(stream.network, "tcp");
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
    assert_eq!(stream.network, "ws");
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
    extensions.insert("fp".to_string(), "chrome".to_string());
    extensions.insert("alpn".to_string(), "h2".to_string());
    extensions.insert("mode".to_string(), "auto".to_string());

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
fn generates_reality_stream_with_settings_and_flow() {
    let mut extensions = std::collections::BTreeMap::new();
    extensions.insert("pbk".to_string(), "test-public-key".to_string());
    extensions.insert("sid".to_string(), "0123abcd".to_string());
    extensions.insert("spx".to_string(), "/".to_string());
    extensions.insert("flow".to_string(), "xtls-rprx-vision".to_string());

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
