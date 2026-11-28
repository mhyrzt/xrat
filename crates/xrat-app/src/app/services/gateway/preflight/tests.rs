use super::routing::parse_routes;
use super::*;
use crate::app::{
    config::GatewaySettings,
    services::gateway::{GatewayOverrides, resolve},
};

fn address(name: &str, subnet: &str) -> InterfaceAddress {
    InterfaceAddress {
        name: name.into(),
        address: Ipv4Addr::new(192, 168, 1, 2),
        subnet: subnet.into(),
        up: true,
        point_to_point: false,
        candidate: true,
    }
}

#[test]
fn chooses_only_unambiguous_interface_and_subnet() {
    let mut settings = resolve(&GatewaySettings::default(), &GatewayOverrides::default()).unwrap();
    let mut checks = Vec::new();
    select_lan(
        &mut settings,
        &[address("eth0", "192.168.1.0/24")],
        &mut checks,
    );
    assert_eq!(settings.interface.value.as_deref(), Some("eth0"));
    assert_eq!(settings.lan_subnet.value.as_deref(), Some("192.168.1.0/24"));
    assert_eq!(settings.interface.source, "discovery");
    let mut ambiguous = resolve(&GatewaySettings::default(), &GatewayOverrides::default()).unwrap();
    let mut checks = Vec::new();
    select_lan(
        &mut ambiguous,
        &[
            address("eth0", "192.168.1.0/24"),
            address("wlan0", "10.0.0.0/24"),
        ],
        &mut checks,
    );
    assert!(ambiguous.interface.value.is_none());
    assert_eq!(checks[0].state, "failed");
}

#[test]
fn rejects_subnet_mismatch_and_ambiguous_subnets() {
    let mut settings = resolve(
        &GatewaySettings::default(),
        &GatewayOverrides {
            interface: Some("eth0".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let mut checks = Vec::new();
    select_lan(
        &mut settings,
        &[
            address("eth0", "192.168.1.0/24"),
            address("eth0", "10.0.0.0/24"),
        ],
        &mut checks,
    );
    assert!(settings.lan_subnet.value.is_none());
    assert_eq!(checks.last().unwrap().state, "failed");
    settings.lan_subnet.value = Some("172.16.0.0/16".into());
    let mut checks = Vec::new();
    select_lan(
        &mut settings,
        &[address("eth0", "192.168.1.0/24")],
        &mut checks,
    );
    assert_eq!(checks.last().unwrap().state, "failed");
}

#[test]
fn parses_main_table_default_routes_and_rejects_bad_data() {
    let input = "Iface Destination Gateway Flags RefCnt Use Metric Mask\neth0 00000000 0101A8C0 0003 0 0 100 00000000\n";
    let routes = parse_default_routes(input).unwrap();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].gateway, Ipv4Addr::new(192, 168, 1, 1));
    assert_eq!(routes[0].metric, 100);
    assert!(parse_default_routes("header\ninvalid route\n").is_err());
}

#[test]
fn upstream_topology_handles_same_interface_and_ambiguous_defaults() {
    let route = |name: &str| DefaultRoute {
        interface: name.into(),
        gateway: Ipv4Addr::new(192, 168, 1, 1),
        metric: 100,
    };
    let shared = inspect_upstream(
        &[route("eth0")],
        &[address("eth0", "192.168.1.0/24")],
        Some("eth0"),
    );
    assert_eq!(shared.state, "incomplete");
    assert!(shared.detail.contains("shares the LAN interface"));
    let separate = inspect_upstream(
        &[route("eth1")],
        &[address("eth1", "192.168.1.0/24")],
        Some("eth0"),
    );
    assert!(separate.detail.contains("separate upstream"));
    let ambiguous = inspect_upstream(
        &[route("eth0"), route("eth1")],
        &[address("eth0", "192.168.1.0/24")],
        Some("eth0"),
    );
    assert_eq!(ambiguous.state, "unknown");
    let missing = inspect_upstream(
        &[route("eth1")],
        &[address("eth0", "192.168.1.0/24")],
        Some("eth0"),
    );
    assert_eq!(missing.state, "failed");
}

#[test]
fn parses_iproute2_tables_and_rejects_invalid_output() {
    let routes = parse_routes(
        br#"[{"dst":"default","gateway":"192.168.1.1","dev":"eth0","table":100,"metric":10}]"#,
    )
    .unwrap();
    assert_eq!(routes[0].table, Some(serde_json::json!(100)));
    assert_eq!(routes[0].gateway.as_deref(), Some("192.168.1.1"));
    assert!(parse_routes(b"invalid").is_err());
}

#[test]
fn caller_capabilities_distinguish_no_new_privileges_from_effective_capability() {
    let check = caller_privileges("CapEff:\t0000000000001000\nNoNewPrivs:\t1\n");
    assert_eq!(check.state, "incomplete");
    assert!(check.detail.contains("has effective CAP_NET_ADMIN"));
    assert!(check.detail.contains("NoNewPrivs=1"));
    assert_eq!(caller_privileges("CapEff: invalid\n").state, "unknown");
    assert!(
        caller_privileges("CapEff: 0\nNoNewPrivs: 0\n")
            .detail
            .contains("does not have")
    );
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn runtime_owner_observes_socket_peer_without_claiming_identity() {
    let dir = tempfile::tempdir().unwrap();
    let socket = crate::app::daemon::ipc::default_socket_path(dir.path());
    let _listener = tokio::net::UnixListener::bind(socket).unwrap();
    let observed = inspect_runtime_owner(Some(dir.path())).await;
    assert_eq!(observed.state, "incomplete");
    assert!(
        observed
            .detail
            .contains(&format!("PID {}", std::process::id()))
    );
    assert!(
        observed
            .detail
            .contains("identity and engine capability inheritance are not verified")
    );
    let missing = tempfile::tempdir().unwrap();
    assert_eq!(
        inspect_runtime_owner(Some(missing.path())).await.state,
        "unknown"
    );
}

#[test]
fn backend_preference_does_not_claim_gateway_support() {
    assert_eq!(inspect_engine("v2ray", None).state, "failed");
    assert_eq!(inspect_engine("xray", None).state, "unknown");
    let missing = inspect_engine(
        "sing-box",
        Some(std::path::Path::new("/xrat-test-missing-engine")),
    );
    assert_eq!(missing.state, "incomplete");
    assert!(
        missing
            .detail
            .contains("selected proxy backend is not known")
    );
}

#[cfg(unix)]
#[test]
fn xray_version_gate_never_claims_gateway_readiness() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("xray");
    std::fs::write(&binary, "#!/bin/sh\nprintf 'Xray 26.7.11\\n'\n").unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
    let report = inspect_engine("xray", Some(&binary));
    assert_eq!(report.state, "incomplete");
    assert!(report.detail.contains("26.7.11"));
    assert!(
        report
            .detail
            .contains("gateway configuration and packet behavior are unverified")
    );
    std::fs::write(&binary, "#!/bin/sh\nprintf 'Xray 26.3.27\\n'\n").unwrap();
    let old = inspect_engine("xray", Some(&binary));
    assert_eq!(old.state, "incomplete");
    assert!(old.detail.contains("did not pass"));
}

#[cfg(unix)]
#[test]
fn singbox_version_gate_is_independent_and_incomplete() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let binary = dir.path().join("sing-box");
    std::fs::write(&binary, "#!/bin/sh\nprintf '1.13.21\\n'\n").unwrap();
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
    let report = inspect_engine("sing-box", Some(&binary));
    assert_eq!(report.state, "incomplete");
    assert!(report.detail.contains("1.13.21"));
    assert!(report.detail.contains("packet behavior are unverified"));
    std::fs::write(&binary, "#!/bin/sh\nprintf '1.12.0\\n'\n").unwrap();
    let old = inspect_engine("sing-box", Some(&binary));
    assert_eq!(old.state, "incomplete");
    assert!(old.detail.contains("did not pass"));
}
