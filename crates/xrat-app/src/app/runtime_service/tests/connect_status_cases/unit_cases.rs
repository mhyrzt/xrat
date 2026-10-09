use super::*;
use crate::app::config::DnsHostValue;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[tokio::test]
async fn xray_preflight_uses_json_config_filename() {
    let mut context = test_context().await;
    let config = imported_config(&context, test_node()).await;
    let validator = context.runtime_paths.root_dir.join("xray-validator.py");
    fs::write(
        &validator,
        r#"#!/usr/bin/env python3
import json
import sys

config_path = sys.argv[sys.argv.index("-c") + 1]
if not config_path.endswith(".json"):
    print(f"failed to get format of config file: {config_path}", file=sys.stderr)
    sys.exit(23)
with open(config_path, "r", encoding="utf-8") as config_file:
    json.load(config_file)
"#,
    )
    .expect("validator should be written");
    let mut permissions = fs::metadata(&validator)
        .expect("validator metadata should load")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&validator, permissions).expect("validator should be executable");
    context.runtime_paths.xray_path = validator;

    let launch = RuntimeService::new(&context)
        .resolve_launch(&config)
        .expect("launch should resolve");

    preflight_runtime(&launch, &context.runtime_paths.runtime_dir)
        .expect("Xray preflight should receive a JSON filename");
}

#[test]
fn running_session_with_unreachable_inbound_is_degraded() {
    let session = runtime_session_with_status(RuntimeSessionStatus::Running);
    let health = RuntimeInboundHealth {
        socks: Some(RuntimeEndpointHealth {
            endpoint: RuntimeEndpoint {
                host: "127.0.0.1".to_string(),
                port: 1080,
            },
            state: RuntimeEndpointState::Unreachable,
        }),
        http: None,
        shadowsocks: None,
    };

    assert_eq!(
        runtime_status_label(&Some(session), &ActiveSessionState::None, true, &health),
        RuntimeSessionDisplay::Degraded
    );
}

#[test]
fn running_session_with_reachable_inbounds_keeps_persisted_status() {
    let session = runtime_session_with_status(RuntimeSessionStatus::Running);
    let health = RuntimeInboundHealth {
        socks: Some(RuntimeEndpointHealth {
            endpoint: RuntimeEndpoint {
                host: "127.0.0.1".to_string(),
                port: 1080,
            },
            state: RuntimeEndpointState::Reachable,
        }),
        http: None,
        shadowsocks: None,
    };

    assert_eq!(
        runtime_status_label(&Some(session), &ActiveSessionState::None, true, &health),
        RuntimeSessionDisplay::Persisted(RuntimeSessionStatus::Running)
    );
}

#[test]
fn rejects_unknown_protocol() {
    let record = ConfigRecord {
        id: xrat_model::ConfigId(1),
        r#ref: "ref000000001".to_string(),
        subscription_id: None,
        dedup_key: "key".to_string(),
        protocol: "unknown".to_string(),
        address: "example.com".to_string(),
        port: 443,
        username: None,
        uuid: None,
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: None,
        sni: None,
        host: None,
        path: None,
        name: None,
        raw_config: "raw".to_string(),
        extensions_json: None,
        is_active: false,
        is_enabled: true,
        is_deleted: false,
        deleted_at: None,
        imported_at: "now".to_string(),
        created_at: "now".to_string(),
        updated_at: "now".to_string(),
    };

    assert!(matches!(
        node_from_record(&record),
        Err(xrat_db::DbError::UnsupportedProtocol(_))
    ));
}

#[test]
fn maps_wildcard_bind_hosts_to_loopback_for_readiness() {
    assert_eq!(connect_host_for_bind_host("0.0.0.0"), "127.0.0.1");
    assert_eq!(connect_host_for_bind_host("::"), "::1");
    assert_eq!(connect_host_for_bind_host("127.0.0.1"), "127.0.0.1");
}

#[tokio::test]
async fn hy2_launch_uses_configured_xray_runtime() {
    let context = test_context().await;
    let config = imported_config(&context, hy2_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&config)
        .expect("hy2 launch should resolve");

    assert_eq!(launch.binary_path, context.runtime_paths.xray_path);
    assert_eq!(launch.ready_port, context.app_config.runtime.socks.port);
    assert!(matches!(launch.config, RuntimeLaunchConfig::Xray(_)));
    assert_eq!(
        launch.endpoints.socks,
        Some(RuntimeEndpoint {
            host: context.app_config.runtime.socks.host.clone(),
            port: context.app_config.runtime.socks.port,
        })
    );
}

#[tokio::test]
async fn hy2_launch_uses_configured_singbox_runtime() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    let config = imported_config(&context, hy2_node()).await;
    let launch = RuntimeService::new(&context)
        .resolve_launch(&config)
        .unwrap();
    assert_eq!(launch.binary_path, context.runtime_paths.sing_box_path);
    assert!(matches!(launch.config, RuntimeLaunchConfig::Singbox(_)));
}

#[tokio::test]
async fn singbox_launch_rejects_socks_udp_disabled() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    context.app_config.runtime.socks.udp = false;
    let config = imported_config(&context, hy2_node()).await;

    let error = match RuntimeService::new(&context).resolve_launch(&config) {
        Ok(_) => panic!("sing-box cannot disable SOCKS UDP independently"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("runtime.socks].udp = false"));
}

#[tokio::test]
async fn singbox_launch_rejects_non_loopback_clash_api_and_port_collisions() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    context.app_config.runtime.stats.enabled = true;
    context.app_config.runtime.stats.host = "0.0.0.0".to_string();
    let config = imported_config(&context, hy2_node()).await;

    let error = match RuntimeService::new(&context).resolve_launch(&config) {
        Ok(_) => panic!("non-loopback Clash API must be rejected"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("beyond loopback"));

    context.app_config.runtime.stats.host = "127.0.0.1".to_string();
    context.app_config.runtime.stats.port = context.app_config.runtime.socks.port;
    let error = match RuntimeService::new(&context).resolve_launch(&config) {
        Ok(_) => panic!("Clash API/socks port collision must be rejected"),
        Err(error) => error,
    };
    assert!(
        error
            .to_string()
            .contains("collides with [runtime.socks].port")
    );
}

#[tokio::test]
async fn hy2_launch_rejects_configured_v2ray_runtime() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "v2ray".to_string();
    let config = imported_config(&context, hy2_node()).await;
    let error = match RuntimeService::new(&context).resolve_launch(&config) {
        Ok(_) => panic!("V2Ray should reject Hy2"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("V2Ray does not support it"));
}

#[tokio::test]
async fn managed_xray_launch_applies_configured_routing() {
    let mut context = test_context().await;
    context.app_config.routing.direct.domain = vec!["domain:direct.example".to_string()];
    context.app_config.routing.block.ip = vec!["203.0.113.0/24".to_string()];
    let imported = imported_config(&context, test_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&imported)
        .expect("Xray launch should resolve");
    let RuntimeLaunchConfig::Xray(config) = launch.config else {
        panic!("expected an Xray runtime config");
    };
    let value = serde_json::to_value(config).expect("config should serialize");

    assert_eq!(value["routing"]["rules"][0]["outboundTag"], "api");
    assert_eq!(value["routing"]["rules"][1]["outboundTag"], "direct");
    assert_eq!(
        value["routing"]["rules"][1]["domain"][0],
        "domain:direct.example"
    );
    assert_eq!(value["routing"]["rules"][2]["outboundTag"], "block");
    assert_eq!(value["routing"]["rules"][2]["ip"][0], "203.0.113.0/24");
}

#[tokio::test]
async fn managed_xray_launch_applies_configured_dns() {
    let mut context = test_context().await;
    context.app_config.dns.query_strategy = "UseIPv4".to_string();
    context.app_config.dns.servers = vec!["8.8.8.8".to_string()];
    context.app_config.dns.use_system_hosts = false;
    context.app_config.dns.hosts.insert(
        "full:example.test".to_string(),
        DnsHostValue::One("192.0.2.10".to_string()),
    );
    let imported = imported_config(&context, test_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&imported)
        .expect("Xray launch should resolve with DNS");
    let RuntimeLaunchConfig::Xray(config) = launch.config else {
        panic!("expected an Xray runtime config");
    };
    let value = serde_json::to_value(config).expect("config should serialize");

    assert_eq!(value["dns"]["queryStrategy"], "UseIPv4");
    assert_eq!(value["dns"]["servers"], serde_json::json!(["8.8.8.8"]));
    assert_eq!(value["dns"]["hosts"]["full:example.test"], "192.0.2.10");
}

#[tokio::test]
async fn managed_singbox_launch_applies_configured_dns() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    context.app_config.dns.query_strategy = "UseIPv4".to_string();
    context.app_config.dns.servers = vec!["8.8.8.8".to_string()];
    context.app_config.dns.use_system_hosts = false;
    context.app_config.dns.hosts.insert(
        "full:example.test".to_string(),
        DnsHostValue::One("192.0.2.10".to_string()),
    );
    let imported = imported_config(&context, hy2_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&imported)
        .expect("sing-box launch should resolve with DNS");
    let RuntimeLaunchConfig::Singbox(config) = launch.config else {
        panic!("expected a sing-box runtime config");
    };
    let value = serde_json::to_value(config).expect("config should serialize");

    assert_eq!(value["dns"]["strategy"], "ipv4_only");
    assert_eq!(value["dns"]["final"], "xrat-dns-0");
    assert_eq!(value["dns"]["servers"][1]["type"], "hosts");
    assert_eq!(
        value["dns"]["rules"][0]["domain"],
        serde_json::json!(["example.test"])
    );
}

#[tokio::test]
async fn configured_singbox_supports_vless_runtime_generation() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    let config = imported_config(&context, test_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&config)
        .expect("VLESS sing-box launch");
    let RuntimeLaunchConfig::Singbox(config) = launch.config else {
        panic!("expected sing-box config");
    };
    assert_eq!(config.outbounds[0]["type"], "vless");
}

#[tokio::test]
async fn managed_singbox_launch_adds_tun_inbound_and_route() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.strict_route = true;
    let config = imported_config(&context, test_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&config)
        .expect("TUN launch should resolve");
    let RuntimeLaunchConfig::Singbox(config) = launch.config else {
        panic!("expected sing-box config");
    };
    let value = serde_json::to_value(config).expect("config should serialize");

    let tun = value["inbounds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|inbound| inbound["type"] == "tun")
        .expect("tun inbound should be present");
    assert_eq!(tun["interface_name"], "xrat0");
    assert_eq!(tun["strict_route"], true);
    assert_eq!(value["route"]["auto_detect_interface"], true);
    assert_eq!(value["route"]["final"], "proxy");
}

#[tokio::test]
async fn managed_xray_launch_adds_tun_inbound() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let config = imported_config(&context, test_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&config)
        .expect("xray TUN launch should resolve");
    let RuntimeLaunchConfig::Xray(config) = launch.config else {
        panic!("expected an xray runtime config");
    };
    let value = serde_json::to_value(config).expect("config should serialize");

    let tun = value["inbounds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|inbound| inbound["protocol"] == "tun")
        .expect("tun inbound should be present");
    assert_eq!(tun["settings"]["name"], "xrat0");
    assert_eq!(
        tun["settings"]["autoSystemRoutingTable"],
        serde_json::json!(["0.0.0.0/1", "128.0.0.0/1"])
    );
    assert!(tun.get("port").is_none());
    assert!(tun.get("listen").is_none());

    let dns_out = value["outbounds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|outbound| outbound["tag"] == "dns-out")
        .expect("dns-out outbound should be present");
    assert_eq!(dns_out["protocol"], "dns");

    let dns_rule = value["routing"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .find(|rule| rule["outboundTag"] == "dns-out")
        .expect("dns routing rule should be present");
    assert_eq!(dns_rule["port"], "53");
    assert_eq!(dns_rule["network"], "tcp,udp");
    assert_eq!(dns_rule["inboundTag"], serde_json::json!(["tun-in"]));
    assert!(value["dns"]["servers"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn managed_xray_launch_adds_tun_ipv6_routes() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.address = vec!["fd00::1/126".to_string()];
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let config = imported_config(&context, test_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&config)
        .expect("xray TUN launch should resolve");
    let RuntimeLaunchConfig::Xray(config) = launch.config else {
        panic!("expected an xray runtime config");
    };
    let value = serde_json::to_value(config).expect("config should serialize");
    let tun = value["inbounds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|inbound| inbound["protocol"] == "tun")
        .expect("tun inbound should be present");
    assert_eq!(
        tun["settings"]["autoSystemRoutingTable"],
        serde_json::json!(["::/1", "8000::/1"])
    );
}

#[tokio::test]
async fn managed_xray_launch_adds_tun_dual_stack_routes() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.address =
        vec!["172.19.0.1/30".to_string(), "fd00::1/126".to_string()];
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let config = imported_config(&context, test_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&config)
        .expect("xray TUN launch should resolve");
    let RuntimeLaunchConfig::Xray(config) = launch.config else {
        panic!("expected an xray runtime config");
    };
    let value = serde_json::to_value(config).expect("config should serialize");
    let tun = value["inbounds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|inbound| inbound["protocol"] == "tun")
        .expect("tun inbound should be present");
    assert_eq!(
        tun["settings"]["autoSystemRoutingTable"],
        serde_json::json!(["0.0.0.0/1", "128.0.0.0/1", "::/1", "8000::/1"])
    );
}

#[tokio::test]
async fn managed_xray_launch_omits_routes_when_auto_route_false() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.auto_route = false;
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let config = imported_config(&context, test_node()).await;
    let service = RuntimeService::new(&context);

    let launch = service
        .resolve_launch(&config)
        .expect("xray TUN launch should resolve");
    let RuntimeLaunchConfig::Xray(config) = launch.config else {
        panic!("expected an xray runtime config");
    };
    let value = serde_json::to_value(config).expect("config should serialize");
    let tun = value["inbounds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|inbound| inbound["protocol"] == "tun")
        .expect("tun inbound should be present");
    assert!(tun["settings"].get("autoSystemRoutingTable").is_none());
}

#[tokio::test]
async fn tun_rejects_v2ray_engine() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "v2ray".to_string();
    context.app_config.runtime.tun.enabled = true;
    let config = imported_config(&context, test_node()).await;

    let error = match RuntimeService::new(&context).resolve_launch(&config) {
        Ok(_) => panic!("TUN with the v2ray engine must fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("V2Ray"));
}

struct MockHostResolver {
    ip: std::net::IpAddr,
}
impl xrat_support::net::HostResolver for MockHostResolver {
    fn resolve(&self, _host: &str, _port: u16) -> Option<std::net::IpAddr> {
        Some(self.ip)
    }
}

struct FailingHostResolver;
impl xrat_support::net::HostResolver for FailingHostResolver {
    fn resolve(&self, _host: &str, _port: u16) -> Option<std::net::IpAddr> {
        None
    }
}

#[tokio::test]
async fn managed_xray_launch_resolves_node_domain_and_preserves_sni() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let mut node = test_node();
    node.tls = Some("tls".to_string());
    node.sni = None;
    let config = imported_config(&context, node).await;

    let mock_ip: std::net::IpAddr = "198.51.100.1".parse().unwrap();
    let ports = xrat_support::readiness::RuntimeProcessPorts {
        resolver: std::sync::Arc::new(MockHostResolver { ip: mock_ip }),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    let launch = service
        .resolve_launch(&config)
        .expect("xray TUN launch should resolve");
    let RuntimeLaunchConfig::Xray(config) = launch.config else {
        panic!("expected an xray runtime config");
    };
    let value = serde_json::to_value(config).expect("config should serialize");

    let outbound = &value["outbounds"][0];
    assert_eq!(outbound["settings"]["vnext"][0]["address"], "198.51.100.1");
    assert_eq!(
        outbound["streamSettings"]["tlsSettings"]["serverName"],
        "example.com"
    );
    assert_eq!(value["dns"]["hosts"]["example.com"], "198.51.100.1");
}

#[tokio::test]
async fn managed_xray_launch_rejects_unresolvable_endpoint_for_tun() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let mut node = test_node();
    node.address = "unresolvable.example.com".to_string();
    let config = imported_config(&context, node).await;

    let ports = xrat_support::readiness::RuntimeProcessPorts {
        resolver: std::sync::Arc::new(FailingHostResolver),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    let error = match service.resolve_launch(&config) {
        Ok(_) => panic!("unresolvable proxy endpoint must fail"),
        Err(err) => err,
    };
    assert!(
        error.to_string().contains(
            "failed to resolve proxy endpoint \"unresolvable.example.com\" for TUN capture"
        ),
        "unexpected error: {error}"
    );
}

#[tokio::test]
async fn managed_xray_launch_rejects_unresolvable_dns_provider_for_tun() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.app_config.dns.servers = vec!["https://custom-dns.example.com/dns-query".to_string()];
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let mut node = test_node();
    node.address = "198.51.100.1".to_string();
    let config = imported_config(&context, node).await;

    let ports = xrat_support::readiness::RuntimeProcessPorts {
        resolver: std::sync::Arc::new(FailingHostResolver),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    let error = match service.resolve_launch(&config) {
        Ok(_) => panic!("unresolvable DNS provider must fail"),
        Err(err) => err,
    };
    assert!(
        error.to_string().contains(
            "failed to resolve DNS provider host \"custom-dns.example.com\" for TUN capture"
        ),
        "unexpected error: {error}"
    );
}

fn runtime_session_with_status(status: RuntimeSessionStatus) -> RuntimeSessionRecord {
    RuntimeSessionRecord {
        id: 1,
        config_id: None,
        status,
        socks_host: Some("127.0.0.1".to_string()),
        socks_port: Some(1080),
        http_host: None,
        http_port: None,
        shadowsocks_host: None,
        shadowsocks_port: None,
        process_id: Some(i64::from(std::process::id())),
        failure_reason: None,
        owner_kind: None,
        owner_instance_id: None,
        last_transition_reason_code: None,
        last_transition_reason_detail: None,
        last_transition_origin: None,
        cooldown_until: None,
        last_failed_at: None,
        last_failed_reason_code: None,
        started_at: Some("1".to_string()),
        stopped_at: None,
        created_at: "1".to_string(),
        updated_at: "1".to_string(),
    }
}

#[tokio::test]
async fn xray_tun_rejects_old_core_without_working_tun() {
    let mut context = test_context().await;
    context.app_config.runtime.tun.enabled = true;
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.3.27 (Xray, Penetrates Everything.)");
    let config = imported_config(&context, test_node()).await;

    let error = match RuntimeService::new(&context).resolve_launch(&config) {
        Ok(_) => panic!("Xray 26.3.27 must be rejected for TUN"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("26.7.11"));
}

#[tokio::test]
async fn xray_tun_accepts_core_with_working_tun() {
    let mut context = test_context().await;
    context.app_config.runtime.tun.enabled = true;
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.11 (Xray, Penetrates Everything.)");
    let config = imported_config(&context, test_node()).await;

    let launch = RuntimeService::new(&context)
        .resolve_launch(&config)
        .expect("Xray 26.7.11 should allow TUN");
    assert!(matches!(launch.config, RuntimeLaunchConfig::Xray(_)));
}

#[tokio::test]
async fn xray_tun_rejects_route_exclusions() {
    let mut context = test_context().await;
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.route_exclude_address = vec!["192.168.0.0/16".to_string()];
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let config = imported_config(&context, test_node()).await;

    let error = match RuntimeService::new(&context).resolve_launch(&config) {
        Ok(_) => panic!("xray must reject TUN route exclusions"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("route_exclude_address"));
}

fn write_fake_xray_version(context: &AppContext, banner: &str) -> std::path::PathBuf {
    let path = context.runtime_paths.root_dir.join("fake-xray-version.sh");
    fs::write(&path, format!("#!/bin/sh\necho \"{banner}\"\n")).expect("script should write");
    let mut permissions = fs::metadata(&path)
        .expect("script metadata should load")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("script should be executable");
    path
}

async fn imported_config(context: &AppContext, node: xrat_model::Node) -> ConfigRecord {
    context
        .db
        .import_nodes(&test_source(), &[node])
        .await
        .expect("node should import");
    context
        .db
        .list_configs(&ConfigListFilter::default())
        .await
        .expect("configs should list")
        .into_iter()
        .next()
        .expect("import should create config")
}

fn hy2_node() -> xrat_model::Node {
    xrat_model::Node {
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
        raw_config: "hy2://secret@hy2.example.com:443?sni=edge.example.com#hy2".to_string(),
    }
}

#[derive(Default)]
struct MockTunOps {
    interfaces:
        std::sync::Mutex<std::collections::HashMap<String, xrat_support::net::KernelInterfaceInfo>>,
    deleted: std::sync::Mutex<Vec<(String, u32)>>,
}

impl xrat_support::net::TunInterfaceOps for MockTunOps {
    fn inspect_interface(
        &self,
        name: &str,
    ) -> std::io::Result<Option<xrat_support::net::KernelInterfaceInfo>> {
        let lock = self.interfaces.lock().unwrap();
        Ok(lock.get(name).cloned())
    }

    fn delete_interface(&self, name: &str, expected_ifindex: u32) -> std::io::Result<()> {
        let mut lock = self.interfaces.lock().unwrap();
        lock.remove(name);
        self.deleted
            .lock()
            .unwrap()
            .push((name.to_string(), expected_ifindex));
        Ok(())
    }
}

#[tokio::test]
async fn tun_cleanup_refuses_foreign_non_tun_interface() {
    let mut context = test_context().await;
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.interface_name = "lo".to_string();

    let mock_tun = std::sync::Arc::new(MockTunOps::default());
    mock_tun.interfaces.lock().unwrap().insert(
        "lo".to_string(),
        xrat_support::net::KernelInterfaceInfo {
            name: "lo".to_string(),
            ifindex: 1,
            is_tun: false,
        },
    );

    let ports = xrat_support::readiness::RuntimeProcessPorts {
        tun: mock_tun.clone(),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    let error = service.cleanup_stale_tun_interface().unwrap_err();
    assert!(error.to_string().contains("not a TUN device"));
    assert!(mock_tun.deleted.lock().unwrap().is_empty());
    assert!(mock_tun.interfaces.lock().unwrap().contains_key("lo"));
}

#[tokio::test]
async fn tun_cleanup_refuses_unowned_tun_interface() {
    let mut context = test_context().await;
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.interface_name = "xrat0".to_string();

    let mock_tun = std::sync::Arc::new(MockTunOps::default());
    mock_tun.interfaces.lock().unwrap().insert(
        "xrat0".to_string(),
        xrat_support::net::KernelInterfaceInfo {
            name: "xrat0".to_string(),
            ifindex: 42,
            is_tun: true,
        },
    );

    let ports = xrat_support::readiness::RuntimeProcessPorts {
        tun: mock_tun.clone(),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    let error = service.cleanup_stale_tun_interface().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("ownership by XRAT could not be verified")
    );
    assert!(mock_tun.deleted.lock().unwrap().is_empty());
    assert!(mock_tun.interfaces.lock().unwrap().contains_key("xrat0"));
}

#[tokio::test]
async fn tun_cleanup_refuses_ifindex_mismatch() {
    let mut context = test_context().await;
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.interface_name = "xrat0".to_string();

    crate::app::runtime_service::tun_ownership::save_ownership(
        &context.runtime_paths.runtime_dir,
        &crate::app::runtime_service::tun_ownership::TunOwnershipRecord {
            interface_name: "xrat0".to_string(),
            ifindex: Some(42),
            session_id: 1,
            engine: "xray".to_string(),
            policy_rules: vec![],
        },
    )
    .unwrap();

    let mock_tun = std::sync::Arc::new(MockTunOps::default());
    mock_tun.interfaces.lock().unwrap().insert(
        "xrat0".to_string(),
        xrat_support::net::KernelInterfaceInfo {
            name: "xrat0".to_string(),
            ifindex: 99,
            is_tun: true,
        },
    );

    let ports = xrat_support::readiness::RuntimeProcessPorts {
        tun: mock_tun.clone(),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    let error = service.cleanup_stale_tun_interface().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("does not match previously recorded interface index")
    );
    assert!(mock_tun.deleted.lock().unwrap().is_empty());
}

#[tokio::test]
async fn tun_cleanup_removes_verified_stale_xrat_tun_interface() {
    let mut context = test_context().await;
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.interface_name = "xrat0".to_string();

    crate::app::runtime_service::tun_ownership::save_ownership(
        &context.runtime_paths.runtime_dir,
        &crate::app::runtime_service::tun_ownership::TunOwnershipRecord {
            interface_name: "xrat0".to_string(),
            ifindex: Some(42),
            session_id: 1,
            engine: "xray".to_string(),
            policy_rules: vec![],
        },
    )
    .unwrap();

    let mock_tun = std::sync::Arc::new(MockTunOps::default());
    mock_tun.interfaces.lock().unwrap().insert(
        "xrat0".to_string(),
        xrat_support::net::KernelInterfaceInfo {
            name: "xrat0".to_string(),
            ifindex: 42,
            is_tun: true,
        },
    );

    let ports = xrat_support::readiness::RuntimeProcessPorts {
        tun: mock_tun.clone(),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    service
        .cleanup_stale_tun_interface()
        .expect("cleanup should succeed");
    assert_eq!(
        mock_tun.deleted.lock().unwrap().as_slice(),
        &[("xrat0".to_string(), 42)]
    );
    assert!(!mock_tun.interfaces.lock().unwrap().contains_key("xrat0"));
    assert_eq!(
        crate::app::runtime_service::tun_ownership::load_ownership(
            &context.runtime_paths.runtime_dir
        ),
        None
    );
}

#[tokio::test]
async fn tun_replacement_preserves_running_session_when_preflight_fails() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.replace_active_session = true;
    context.app_config.runtime.tun.enabled = true;
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");

    let cfg1 = imported_config(&context, test_node()).await;
    let cfg2 = imported_config(&context, hy2_node()).await;

    let _session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(cfg1.id),
            status: RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(10808),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(i64::from(std::process::id())),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .unwrap();
    context.db.set_active_config(cfg1.id).await.unwrap();

    use std::os::unix::process::ExitStatusExt;
    struct FailingPreflightSpawner;
    #[async_trait::async_trait]
    impl xrat_support::process::ProcessSpawner for FailingPreflightSpawner {
        fn spawn(
            &self,
            _spec: &xrat_support::process::CommandSpec,
        ) -> std::io::Result<xrat_support::process::Child> {
            unimplemented!()
        }
        fn run(
            &self,
            spec: &xrat_support::process::CommandSpec,
            _capture: bool,
        ) -> std::io::Result<std::process::Output> {
            let is_version = spec
                .args
                .iter()
                .any(|a| a.to_string_lossy().contains("version"));
            if is_version {
                Ok(std::process::Output {
                    status: std::process::ExitStatus::from_raw(0),
                    stdout: b"Xray 26.7.28\n".to_vec(),
                    stderr: Vec::new(),
                })
            } else {
                Ok(std::process::Output {
                    status: std::process::ExitStatus::from_raw(1 << 8),
                    stdout: Vec::new(),
                    stderr: b"config invalid\n".to_vec(),
                })
            }
        }
        async fn output_async(
            &self,
            spec: &xrat_support::process::CommandSpec,
        ) -> std::io::Result<std::process::Output> {
            self.run(spec, true)
        }
    }

    let ports = xrat_support::readiness::RuntimeProcessPorts {
        spawner: std::sync::Arc::new(FailingPreflightSpawner),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    let result = service.connect(ConnectRequest { config_id: cfg2.id }).await;
    assert!(result.is_err());

    let session = context
        .db
        .get_running_runtime_session()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.status, RuntimeSessionStatus::Running);
    assert_eq!(
        context.db.get_active_config().await.unwrap().map(|c| c.id),
        Some(cfg1.id)
    );
}

#[tokio::test]
async fn tun_connect_replacement_preserves_running_session_when_resolution_fails() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.replace_active_session = true;
    context.app_config.runtime.tun.enabled = true;
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");

    let cfg1 = imported_config(&context, test_node()).await;
    let mut node2 = hy2_node();
    node2.address = "unresolvable.example.com".to_string();
    let summary = context
        .db
        .import_nodes(&test_source(), &[node2])
        .await
        .expect("node should import");
    assert_eq!(summary.imported_configs, 1);
    let cfg2 = context
        .db
        .list_configs(&Default::default())
        .await
        .unwrap()
        .into_iter()
        .find(|c| c.id != cfg1.id)
        .expect("second config should exist");

    let _session_id = context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: Some(cfg1.id),
            status: RuntimeSessionStatus::Running,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(10808),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(i64::from(std::process::id())),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
        })
        .await
        .unwrap();
    context.db.set_active_config(cfg1.id).await.unwrap();

    let ports = xrat_support::readiness::RuntimeProcessPorts {
        resolver: std::sync::Arc::new(FailingHostResolver),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    let result = service.connect(ConnectRequest { config_id: cfg2.id }).await;
    assert!(result.is_err());
    assert!(
        result.unwrap_err().to_string().contains(
            "failed to resolve proxy endpoint \"unresolvable.example.com\" for TUN capture"
        )
    );

    let session = context
        .db
        .get_running_runtime_session()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(session.status, RuntimeSessionStatus::Running);
    assert_eq!(
        context.db.get_active_config().await.unwrap().map(|c| c.id),
        Some(cfg1.id)
    );
}

#[tokio::test]
async fn tun_cleanup_refuses_unknown_ifindex() {
    let mut context = test_context().await;
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.interface_name = "xrat0".to_string();

    crate::app::runtime_service::tun_ownership::save_ownership(
        &context.runtime_paths.runtime_dir,
        &crate::app::runtime_service::tun_ownership::TunOwnershipRecord {
            interface_name: "xrat0".to_string(),
            ifindex: None,
            session_id: 1,
            engine: "xray".to_string(),
            policy_rules: vec![],
        },
    )
    .unwrap();

    let mock_tun = std::sync::Arc::new(MockTunOps::default());
    mock_tun.interfaces.lock().unwrap().insert(
        "xrat0".to_string(),
        xrat_support::net::KernelInterfaceInfo {
            name: "xrat0".to_string(),
            ifindex: 99,
            is_tun: true,
        },
    );

    let ports = xrat_support::readiness::RuntimeProcessPorts {
        tun: mock_tun.clone(),
        ..Default::default()
    };
    let service = RuntimeService::with_process_ports(&context, ports);

    let error = service.cleanup_stale_tun_interface().unwrap_err();
    assert!(error.to_string().contains("no verified kernel index"));
    assert!(mock_tun.deleted.lock().unwrap().is_empty());
}

#[test]
fn tun_dns_bootstrap_parses_supported_server_formats() {
    use crate::app::runtime_service::launch::extract_dns_server_host;
    for server in [
        "https://dns.google/dns-query",
        "https+local://dns.google:8443/dns-query",
        "tcp://dns.google:53",
        "tcp+local://dns.google:53",
        "quic+local://dns.google:853",
        "h2c://dns.google/dns-query",
        "dns.google:53",
    ] {
        assert_eq!(
            extract_dns_server_host(server).unwrap().as_deref(),
            Some("dns.google"),
            "{server}"
        );
    }
    for server in [
        "1.1.1.1",
        "1.1.1.1:53",
        "2606:4700:4700::1111",
        "::1",
        "[2606:4700:4700::1111]:53",
        "https://[2606:4700:4700::1111]/dns-query",
        "https://[2606:4700:4700::1111]:8443/dns-query",
        "tcp+local://[::1]:53",
        "https+local://1.1.1.1/dns-query",
        "localhost",
        "fakedns",
    ] {
        assert_eq!(extract_dns_server_host(server).unwrap(), None, "{server}");
    }
    assert!(extract_dns_server_host("https://[invalid]/dns-query").is_err());
}

#[tokio::test]
async fn managed_xray_launch_skips_resolution_for_dns_ip_literals_and_special_servers() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.app_config.dns.servers = [
        "2606:4700:4700::1111",
        "https://[2606:4700:4700::1111]:8443/dns-query",
        "tcp+local://8.8.8.8:53",
        "localhost",
        "fakedns",
    ]
    .map(str::to_string)
    .to_vec();
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let mut node = test_node();
    node.address = "198.51.100.1".to_string();
    let config = imported_config(&context, node).await;
    struct UnexpectedResolver;
    impl xrat_support::net::HostResolver for UnexpectedResolver {
        fn resolve(&self, host: &str, _: u16) -> Option<std::net::IpAddr> {
            panic!("unexpected bootstrap resolution of {host}");
        }
    }
    let ports = xrat_support::readiness::RuntimeProcessPorts {
        resolver: std::sync::Arc::new(UnexpectedResolver),
        ..Default::default()
    };
    RuntimeService::with_process_ports(&context, ports)
        .resolve_launch(&config)
        .expect("literal and special DNS servers must not require host resolution");
}

#[tokio::test]
async fn managed_xray_tun_launch_applies_split_blacklist_and_whitelist() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "xray".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.split_mode = crate::app::config::TunSplitMode::Blacklist;
    context.app_config.runtime.tun.blacklist =
        vec!["firefox".to_string(), "/usr/bin/curl".to_string()];
    context.runtime_paths.xray_path =
        write_fake_xray_version(&context, "Xray 26.7.28 (Xray, Penetrates Everything.)");
    let config = imported_config(&context, test_node()).await;

    let launch = RuntimeService::new(&context)
        .resolve_launch(&config)
        .expect("xray TUN blacklist launch should resolve");
    let RuntimeLaunchConfig::Xray(xray_cfg) = launch.config else {
        panic!("expected Xray config");
    };
    let value = serde_json::to_value(xray_cfg).expect("config should serialize");
    let rules = value["routing"]["rules"].as_array().unwrap();
    assert_eq!(rules[0]["outboundTag"], "api");
    assert_eq!(rules[1]["inboundTag"], serde_json::json!(["tun-in"]));
    assert_eq!(
        rules[1]["process"],
        serde_json::json!(["firefox", "/usr/bin/curl"])
    );
    assert_eq!(rules[1]["outboundTag"], "direct");

    context.app_config.runtime.tun.split_mode = crate::app::config::TunSplitMode::Whitelist;
    context.app_config.runtime.tun.whitelist = vec!["telegram-desktop".to_string()];
    let launch = RuntimeService::new(&context)
        .resolve_launch(&config)
        .expect("xray TUN whitelist launch should resolve");
    let RuntimeLaunchConfig::Xray(xray_cfg) = launch.config else {
        panic!("expected Xray config");
    };
    let value = serde_json::to_value(xray_cfg).expect("config should serialize");
    let rules = value["routing"]["rules"].as_array().unwrap();
    assert_eq!(rules[0]["outboundTag"], "api");
    assert_eq!(rules[1]["inboundTag"], serde_json::json!(["tun-in"]));
    assert_eq!(rules[1]["process"], serde_json::json!(["self/"]));
    assert_eq!(rules[1]["outboundTag"], "direct");
    assert_eq!(rules[2]["inboundTag"], serde_json::json!(["tun-in"]));
    assert_eq!(rules[2]["process"], serde_json::json!(["telegram-desktop"]));
    assert_eq!(rules[2]["outboundTag"], "proxy");
    assert_eq!(rules[3]["inboundTag"], serde_json::json!(["tun-in"]));
    assert_eq!(rules[3]["outboundTag"], "direct");
}

#[tokio::test]
async fn managed_singbox_tun_launch_applies_split_blacklist_and_whitelist() {
    let mut context = test_context().await;
    context.app_config.runtime.engine = "sing-box".to_string();
    context.app_config.runtime.tun.enabled = true;
    context.app_config.runtime.tun.split_mode = crate::app::config::TunSplitMode::Blacklist;
    context.app_config.runtime.tun.blacklist = vec![
        "firefox".to_string(),
        "/usr/bin/curl".to_string(),
        "/opt/discord/".to_string(),
    ];
    let config = imported_config(&context, test_node()).await;

    let launch = RuntimeService::new(&context)
        .resolve_launch(&config)
        .expect("sing-box TUN blacklist launch should resolve");
    let RuntimeLaunchConfig::Singbox(sb_cfg) = launch.config else {
        panic!("expected sing-box config");
    };
    let value = serde_json::to_value(sb_cfg).expect("config should serialize");
    let rules = value["route"]["rules"].as_array().unwrap();
    assert_eq!(rules[0]["action"], "sniff");
    assert_eq!(rules[1]["ip_is_private"], true);
    assert_eq!(rules[2]["inbound"], serde_json::json!(["tun-in"]));
    assert_eq!(rules[2]["process_name"], serde_json::json!(["firefox"]));
    assert_eq!(
        rules[2]["process_path"],
        serde_json::json!(["/usr/bin/curl"])
    );
    assert_eq!(
        rules[2]["process_path_regex"],
        serde_json::json!(["^/opt/discord/.*"])
    );
    assert_eq!(rules[2]["outbound"], "direct");

    context.app_config.runtime.tun.split_mode = crate::app::config::TunSplitMode::Whitelist;
    context.app_config.runtime.tun.whitelist = vec!["telegram-desktop".to_string()];
    let launch = RuntimeService::new(&context)
        .resolve_launch(&config)
        .expect("sing-box TUN whitelist launch should resolve");
    let RuntimeLaunchConfig::Singbox(sb_cfg) = launch.config else {
        panic!("expected sing-box config");
    };
    let value = serde_json::to_value(sb_cfg).expect("config should serialize");
    let rules = value["route"]["rules"].as_array().unwrap();
    assert_eq!(rules[0]["action"], "sniff");
    assert_eq!(rules[1]["ip_is_private"], true);
    assert_eq!(rules[2]["inbound"], serde_json::json!(["tun-in"]));
    assert_eq!(
        rules[2]["process_name"],
        serde_json::json!(["telegram-desktop"])
    );
    assert_eq!(rules[2]["outbound"], "proxy");
    assert_eq!(rules[3]["inbound"], serde_json::json!(["tun-in"]));
    assert_eq!(rules[3]["outbound"], "direct");
    assert_eq!(value["route"]["final"], "proxy");
}
