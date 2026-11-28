use super::*;

#[test]
#[ignore = "requires a disposable Linux network namespace and nftables"]
fn closed_guard_preserves_conflict_and_resumes_installation() {
    use std::process::Command;

    const CHILD: &str = "XRAT_GATEWAY_GUARD_PARENT_NETNS";
    let current = std::fs::read_link("/proc/self/ns/net").unwrap();
    let Some(parent) = std::env::var_os(CHILD) else {
        let status = Command::new("unshare")
                .args(["--user", "--map-root-user", "--net"])
                .arg(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "app::services::gateway::lifecycle::tests::closed_guard_preserves_conflict_and_resumes_installation",
                ])
                .env(CHILD, current)
                .status()
                .unwrap();
        assert!(status.success());
        return;
    };
    assert_ne!(current.as_os_str(), parent);
    for device in ["all", "default", "lo"] {
        std::fs::write(format!("/proc/sys/net/ipv4/conf/{device}/forwarding"), "0").unwrap();
    }

    assert!(
        Command::new("ip")
            .args(["link", "set", "lo", "up"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("ip")
            .args(["address", "add", "192.0.2.1/24", "dev", "lo"])
            .status()
            .unwrap()
            .success()
    );
    let directory = tempfile::tempdir().unwrap();
    let intent = journal::InstallIntent {
        installation_id: uuid::Uuid::new_v4(),
        lan_interface: "lo".into(),
        lan_ifindex: 1,
        lan_address: std::net::Ipv4Addr::LOCALHOST,
        lan_subnet: "127.0.0.0/8".into(),
        local_subnets: vec!["192.0.2.0/24".into()],
        previous_ipv4_forwarding: false,
        boot_id: None,
    };
    let apply = xrat_support::net::firewall::nftables::apply_transaction;
    apply("create table ip xrat_gateway { comment \"foreign\"; }\n").unwrap();
    assert!(verify_install_guard(&intent).is_err());
    assert!(establish_closed_guard(directory.path(), &intent).is_err());
    assert_eq!(
        journal::load(directory.path()).unwrap().as_ref(),
        Some(&intent)
    );

    let observed = Command::new("nft")
        .args(["list", "table", "ip", "xrat_gateway"])
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&observed.stdout).contains("foreign"));

    apply("delete table ip xrat_gateway\n").unwrap();
    establish_closed_guard(directory.path(), &intent).unwrap();
    establish_closed_guard(directory.path(), &intent).unwrap();
    assert!(firewall::inspect_closed(&intent).unwrap());

    assert!(
        Command::new("ip")
            .args(["link", "add", "fake-capture", "type", "dummy"])
            .status()
            .unwrap()
            .success()
    );
    use xrat_support::net::{SystemTunInterfaceOps, TunInterfaceOps};
    let capture = SystemTunInterfaceOps
        .inspect_interface("fake-capture")
        .unwrap()
        .unwrap();
    let capture_index = std::num::NonZeroU32::new(capture.ifindex).unwrap();
    let open = xrat_support::net::firewall::gateway::capture_transaction(
        std::num::NonZeroU32::new(intent.lan_ifindex).unwrap(),
        capture_index,
        "127.0.0.0".parse().unwrap(),
        8,
    )
    .unwrap();
    apply(&open).unwrap();
    assert!(!firewall::inspect_closed(&intent).unwrap());
    firewall::close_capture(&intent, "fake-capture", capture_index).unwrap();
    assert!(firewall::inspect_closed(&intent).unwrap());
    apply(&open).unwrap();
    assert!(
        Command::new("ip")
            .args(["link", "delete", "fake-capture"])
            .status()
            .unwrap()
            .success()
    );
    firewall::close_capture(&intent, "fake-capture", capture_index).unwrap();
    assert!(firewall::inspect_closed(&intent).unwrap());
    let mut stale_intent = journal::load(directory.path()).unwrap().unwrap();
    stale_intent.lan_ifindex += 100;
    journal::replace(directory.path(), &stale_intent).unwrap();
    let previous_boot = CaptureIntent {
        boot_id: uuid::Uuid::new_v4(),
        installation_id: intent.installation_id,
        session_id: 42,
        tun_name: "fake-capture".into(),
        tun_index: capture_index,
        lan_forwarding: InterfaceForwarding::inspect("lo").unwrap(),
        capture_forwarding: InterfaceForwarding::inspect("lo").unwrap(),
    };
    journal::create_record(directory.path(), CAPTURE_FILE, &previous_boot).unwrap();
    apply("delete table ip xrat_gateway\n").unwrap();
    let forwarding = "/proc/sys/net/ipv4/conf/lo/forwarding";
    std::fs::write(forwarding, "1").unwrap();
    assert!(recover_boot(directory.path()).is_err());
    assert!(directory.path().join(CAPTURE_FILE).exists());
    std::fs::write(forwarding, "0").unwrap();
    recover_boot(directory.path()).unwrap();
    let rebound = journal::load(directory.path()).unwrap().unwrap();
    assert_eq!(rebound.lan_ifindex, intent.lan_ifindex);
    assert_eq!(rebound.boot_id, Some(boot_id().unwrap()));
    assert!(!directory.path().join(CAPTURE_FILE).exists());
    assert!(firewall::inspect_closed(&intent).unwrap());
    apply("add rule ip xrat_gateway forward counter accept\n").unwrap();
    assert!(remove(directory.path(), true).is_err());
    assert!(remove(directory.path(), false).is_err());
    assert!(journal::load(directory.path()).unwrap().is_some());
    apply("delete table ip xrat_gateway\n").unwrap();
    establish_closed_guard(directory.path(), &rebound).unwrap();
    remove(directory.path(), true).unwrap();
    assert!(firewall::inspect_closed(&intent).unwrap());
    remove(directory.path(), false).unwrap();
    assert!(journal::load(directory.path()).unwrap().is_none());
    assert!(!firewall::inspect_closed(&intent).unwrap());
    remove(directory.path(), false).unwrap();
}
