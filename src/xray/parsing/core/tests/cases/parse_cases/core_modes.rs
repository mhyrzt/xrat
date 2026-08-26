use super::*;

#[test]
fn test_parse_minimal_config_loose() {
    let json = r#"{
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_loose(json).unwrap();
    assert!(config.inbounds.is_some());
    assert!(config.outbounds.is_some());
    assert!(config.log.is_none());
}

#[test]
fn test_parse_minimal_config_strict() {
    let json = r#"{
        "inbounds": [],
        "outbounds": []
    }"#;

    let config = XrayConfig::from_json_strict(json).unwrap();
    assert!(config.inbounds.is_some());
    assert!(config.outbounds.is_some());
}

#[test]
fn test_loose_mode_allows_unknown_fields() {
    let json = r#"{
        "inbounds": [],
        "outbounds": [],
        "unknownField": "should be ignored",
        "anotherUnknown": 123
    }"#;

    let result = XrayConfig::from_json_loose(json);
    assert!(result.is_ok(), "Loose mode should allow unknown fields");
}

#[test]
fn test_strict_mode_rejects_unknown_fields() {
    let json = r#"{
        "inbounds": [],
        "outbounds": [],
        "unknownField": "should cause error"
    }"#;

    let result = XrayConfig::from_json_strict(json);
    assert!(result.is_err(), "Strict mode should reject unknown fields");
}

#[test]
fn test_parse_mode_controls_unknown_fields() {
    let json = r#"{
        "inbounds": [],
        "outbounds": [],
        "unknownField": "allowed outside strict mode"
    }"#;

    assert!(XrayConfig::from_json_with_mode(json, ParseMode::Strict).is_err());
    assert!(XrayConfig::from_json_with_mode(json, ParseMode::Lenient).is_ok());
    assert!(XrayConfig::from_json_with_mode(json, ParseMode::Auto).is_ok());
}

#[test]
fn strict_mode_rejects_nested_unknown_fields_with_paths() {
    let json = r#"{
        "dns": {
            "servers": ["1.1.1.1"],
            "futureDnsOption": true
        },
        "outbounds": [{
            "protocol": "freedom",
            "streamSettings": {
                "network": "raw",
                "futureTransportOption": 1
            }
        }]
    }"#;

    let error = XrayConfig::from_json_strict(json).unwrap_err().to_string();
    assert!(error.contains("$.dns.futureDnsOption"), "{error}");
    assert!(
        error.contains("$.outbounds[0].streamSettings.futureTransportOption"),
        "{error}"
    );
}

#[test]
fn loose_mode_round_trips_unknown_root_and_nested_fields() {
    let json = r#"{
        "futureRoot": {"enabled": true},
        "dns": {
            "servers": ["1.1.1.1"],
            "futureDnsOption": {"mode": "fast"}
        },
        "outbounds": [{
            "protocol": "freedom",
            "streamSettings": {
                "network": "raw",
                "futureTransportOption": [1, 2, 3]
            }
        }]
    }"#;

    let parsed = XrayConfig::from_json_loose(json).unwrap();
    let serialized: serde_json::Value = serde_json::from_str(&parsed.to_json().unwrap()).unwrap();

    assert_eq!(serialized["futureRoot"]["enabled"], true);
    assert_eq!(serialized["dns"]["futureDnsOption"]["mode"], "fast");
    assert_eq!(
        serialized["outbounds"][0]["streamSettings"]["futureTransportOption"],
        serde_json::json!([1, 2, 3])
    );
}

#[test]
fn parses_direct_and_legacy_outbound_server_forms() {
    let json = r#"{
        "outbounds": [
            {"protocol":"http","settings":{"address":"proxy.example","port":8080}},
            {"protocol":"socks","settings":{"servers":[{"address":"127.0.0.1","port":1080}]}},
            {"protocol":"shadowsocks","settings":{"servers":[{"address":"ss.example","port":443,"method":"2022-blake3-aes-128-gcm","password":"secret"}]}},
            {"protocol":"trojan","settings":{"servers":[{"address":"trojan.example","port":443,"password":"secret"}]}},
            {"protocol":"vless","settings":{"address":"vless.example","port":443,"id":"00000000-0000-0000-0000-000000000001"}},
            {"protocol":"vmess","settings":{"vnext":[{"address":"vmess.example","port":443,"users":[{"id":"00000000-0000-0000-0000-000000000002","security":"auto"}]}]}}
        ]
    }"#;

    let parsed = XrayConfig::from_json_strict(json).unwrap();
    let serialized: serde_json::Value = serde_json::from_str(&parsed.to_json().unwrap()).unwrap();
    assert_eq!(
        serialized["outbounds"][4]["settings"]["id"],
        "00000000-0000-0000-0000-000000000001"
    );
    assert_eq!(
        serialized["outbounds"][5]["settings"]["vnext"][0]["users"][0]["id"],
        "00000000-0000-0000-0000-000000000002"
    );
}

#[test]
fn parses_outbound_aliases_and_acronym_keys() {
    let json = r#"{
        "outbounds": [
            {"protocol":"block","settings":{"response":{"type":"none"}}},
            {"protocol":"direct","mux":{"xudpProxyUDP443":"reject"},"settings":{"targetStrategy":"UseIP"}},
            {"protocol":"wireguard","settings":{"secretKey":"secret","address":["10.0.0.2/32"],"peers":[{"endpoint":"wg.example:51820","publicKey":"key","allowedIPs":["0.0.0.0/0"]}],"noKernelTun":true}}
        ]
    }"#;

    XrayConfig::from_json_strict(json).unwrap();
}

#[test]
fn strict_inbound_dispatch_validates_settings_and_loose_preserves_extensions() {
    let valid = r#"{
        "inbounds": [
            {"protocol":"mixed","settings":{"auth":"noauth","udp":true}},
            {"protocol":"dokodemo-door","settings":{"address":"1.1.1.1","port":53,"network":"tcp,udp"}},
            {"protocol":"vless","settings":{"users":[{"id":"00000000-0000-0000-0000-000000000001"}],"clients":[{"id":"00000000-0000-0000-0000-000000000002"}],"decryption":"none","flow":"xtls-rprx-vision","fallbacks":[{"type":"unix","dest":"/tmp/xray.sock","xver":1}]}},
            {"protocol":"tun","settings":{"name":"xray0","mtu":1500,"gateway":["10.0.0.1/24"],"dns":["1.1.1.1"],"autoSystemRoutingTable":["main"],"autoOutboundsInterface":"eth0"}}
        ]
    }"#;
    XrayConfig::from_json_strict(valid).unwrap();

    let invalid = r#"{"inbounds":[{"protocol":"socks","settings":{"futureOption":true}}]}"#;
    assert!(XrayConfig::from_json_strict(invalid).is_err());
    let loose = XrayConfig::from_json_loose(invalid).unwrap();
    let value: serde_json::Value = serde_json::from_str(&loose.to_json().unwrap()).unwrap();
    assert_eq!(value["inbounds"][0]["settings"]["futureOption"], true);

    let unsupported =
        r#"{"inbounds":[{"protocol":"future-protocol","settings":{"enabled":true}}]}"#;
    assert!(XrayConfig::from_json_strict(unsupported).is_err());
    assert!(XrayConfig::from_json_loose(unsupported).is_ok());
}

#[test]
fn strict_mode_reports_unknown_inbound_outbound_security_and_routing_fields() {
    let json = r#"{
        "routing":{"rules":[{"type":"field","futureRule":true}]},
        "inbounds":[{"protocol":"socks","settings":{"futureInbound":true}}],
        "outbounds":[{
            "protocol":"freedom",
            "settings":{"futureOutbound":true},
            "streamSettings":{"security":"tls","tlsSettings":{"futureTls":true}}
        }]
    }"#;
    let error = XrayConfig::from_json_strict(json).unwrap_err().to_string();
    for path in [
        "$.routing.rules[0].futureRule",
        "$.inbounds[0].settings.futureInbound",
        "$.outbounds[0].settings.futureOutbound",
        "$.outbounds[0].streamSettings.tlsSettings.futureTls",
    ] {
        assert!(error.contains(path), "missing {path} in {error}");
    }
}

#[test]
fn parses_stable_and_prerelease_core_transport_fields() {
    let json = r#"{
        "fakeDns":{"ipPool":"198.18.0.0/15","poolSize":4294967295},
        "api":{"tag":"api","listen":"127.0.0.1:10085","services":["StatsService","ObservatoryService"]},
        "metrics":{"tag":"metrics","listen":"127.0.0.1:11111"},
        "observatory":{"subjectSelector":["proxy"],"probeURL":"https://example.com","probeInterval":"10s"},
        "env":{"XRAY_LOCATION_ASSET":"/tmp/assets"},
        "geodata":{"cron":"0 0 * * *","outbound":"direct","assets":[{"url":"https://example.com/geoip.dat","file":"geoip.dat"}]},
        "routing":{"rules":[{"type":"field","domains":"example.com","source":"192.0.2.1","network":"tcp,udp"}],"balancers":[{"tag":"balanced","selector":["proxy"],"strategy":{"type":"leastPing","settings":{"expected":4294967295,"maxRTT":"10s"}}}]},
        "outbounds":[{
            "protocol":"freedom",
            "streamSettings":{
                "address":"example.com","port":443,"network":"tcp","method":"websocket",
                "tcpSettings":{"header":{"type":"none"}},
                "splithttpSettings":{"mode":"auto"},
                "grpcSettings":{"authority":"example.com","idle_timeout":10,"health_check_timeout":20,"permit_without_stream":true,"initial_windows_size":65535,"user_agent":"xrat"},
                "wsSettings":{"host":"example.com","acceptProxyProtocol":true,"heartbeatPeriod":30},
                "httpupgradeSettings":{"acceptProxyProtocol":true},
                "kcpSettings":{"seed":"seed","header":{"type":"none"},"cwndMultiplier":2,"maxSendingWindow":128},
                "tlsSettings":{"curvePreferences":["X25519"],"pinnedPeerCertSha256":"pin","verifyPeerCertByName":"example.com","echConfigList":"ech"},
                "realitySettings":{"target":"example.com:443","type":"tcp","password":"key","mldsa65Verify":"pq"},
                "sockopt":{"tcpWindowClamp":4096,"penetrate":true,"tcpMptcp":true,"addressPortStrategy":"UseIP","trustedXForwardedFor":["127.0.0.1"]}
            }
        }]
    }"#;
    let config = XrayConfig::from_json_strict(json).unwrap();
    let value: serde_json::Value = serde_json::from_str(&config.to_json().unwrap()).unwrap();
    assert_eq!(value["fakeDns"]["poolSize"], 4294967295_u64);
    assert_eq!(
        value["outbounds"][0]["streamSettings"]["method"],
        "websocket"
    );
    assert!(
        value["outbounds"][0]["streamSettings"]
            .get("rawSettings")
            .is_some()
    );
    assert!(
        value["outbounds"][0]["streamSettings"]
            .get("xhttpSettings")
            .is_some()
    );
}
