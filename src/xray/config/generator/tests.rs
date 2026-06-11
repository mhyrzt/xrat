use super::{
    enable_stats_api, generate_probe_config, generate_runtime_config,
    generate_runtime_config_for_inbounds,
};
use crate::model::{Node, Protocol};

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
