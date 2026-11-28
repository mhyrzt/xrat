use super::*;

pub fn exercise() {
    process::assert_isolated();
    process::ip(&["link", "add", "gateway-lan", "type", "bridge"]);
    process::ip(&["address", "add", "192.0.2.2/24", "dev", "gateway-lan"]);
    process::ip(&["link", "set", "gateway-lan", "up"]);

    let router = Peer::new("router-port", "192.0.2.1/24", Some("gateway-lan"));
    router.ip(&["address", "add", "198.51.100.1/32", "dev", "lo"]);
    let client = Peer::new("client-port", "192.0.2.10/24", Some("gateway-lan"));
    client.ip(&["route", "add", "default", "via", "192.0.2.2"]);
    process::ip(&["route", "add", "default", "via", "192.0.2.1"]);

    process::checked(
        client
            .command("ping")
            .args(["-c", "1", "-W", "2", "192.0.2.2"]),
    );
    std::fs::write("/proc/sys/net/ipv4/ip_forward", "0").unwrap();
    let ping_router = || {
        client
            .command("ping")
            .args(["-c", "1", "-W", "2", "198.51.100.1"])
            .output()
            .unwrap()
            .status
            .success()
    };
    assert!(
        !ping_router(),
        "off-LAN destination reached with forwarding disabled"
    );
    std::fs::write("/proc/sys/net/ipv4/ip_forward", "1").unwrap();
    for interface in ["all", "gateway-lan"] {
        std::fs::write(
            format!("/proc/sys/net/ipv4/conf/{interface}/send_redirects"),
            "0",
        )
        .unwrap();
    }
    assert!(
        ping_router(),
        "same-interface forwarding did not reach the router"
    );
    // Locate the interface through netlink, since sysfs can be mounted for the parent namespace.
    let links = process::ip(&["-j", "link", "show", "dev", "gateway-lan"]);
    let gateway_index = links[0]["ifindex"].as_u64().unwrap();
    let guard = format!(
        "create table ip xrat_gateway\n\
         add chain ip xrat_gateway forward {{ type filter hook forward priority -10; policy accept; }}\n\
         add rule ip xrat_gateway forward meta iif {gateway_index} counter drop\n"
    );
    let apply = xrat_support::net::firewall::nftables::apply_transaction;
    apply(&guard).unwrap();
    assert!(
        apply(&guard).is_err(),
        "duplicate installation was accepted"
    );
    assert!(
        !ping_router(),
        "client reached the router while the forward guard was active"
    );
    process::checked(
        client
            .command("ping")
            .args(["-c", "1", "-W", "2", "192.0.2.2"]),
    );
    let capture = Peer::new("capture-path", "203.0.113.2/30", None);
    process::ip(&["address", "add", "203.0.113.1/30", "dev", "capture-path"]);
    capture.ip(&["address", "add", "198.51.100.2/32", "dev", "lan"]);
    capture.ip(&["route", "add", "192.0.2.0/24", "via", "203.0.113.1"]);
    for interface in ["all", "capture-path"] {
        std::fs::write(
            format!("/proc/sys/net/ipv4/conf/{interface}/rp_filter"),
            "0",
        )
        .unwrap();
    }
    std::fs::write("/proc/sys/net/ipv4/ip_forward", "0").unwrap();
    use xrat_support::net::gateway_forwarding::InterfaceForwarding;
    let lan_forwarding = InterfaceForwarding::inspect("gateway-lan").unwrap();
    let capture_forwarding = InterfaceForwarding::inspect("capture-path").unwrap();
    lan_forwarding.enable().unwrap();
    capture_forwarding.enable().unwrap();
    assert_eq!(
        std::fs::read_to_string("/proc/sys/net/ipv4/ip_forward")
            .unwrap()
            .trim(),
        "0"
    );
    process::ip(&["address", "add", "203.0.114.2/24", "dev", "gateway-lan"]);
    let local_subnets = vec!["203.0.114.0/24".to_string()];
    let install_routes = || {
        xrat_support::net::gateway_routing::install_with_local(
            "gateway-lan",
            "192.0.2.0/24",
            "capture-path",
            &local_subnets,
        )
    };
    process::ip(&[
        "route",
        "add",
        "blackhole",
        "203.0.113.0/24",
        "table",
        "420",
    ]);
    assert_eq!(
        install_routes().unwrap_err().kind(),
        std::io::ErrorKind::AlreadyExists
    );
    assert_eq!(
        process::ip(&["-j", "route", "show", "table", "420"])[0]["dst"],
        "203.0.113.0/24"
    );
    process::ip(&[
        "route",
        "delete",
        "blackhole",
        "203.0.113.0/24",
        "table",
        "420",
    ]);
    install_routes().unwrap();
    assert_eq!(
        process::ip(&[
            "-j",
            "route",
            "get",
            "203.0.114.1",
            "from",
            "192.0.2.10",
            "iif",
            "gateway-lan"
        ])[0]["dev"],
        "gateway-lan"
    );
    let verify_replacement = || {
        xrat_support::net::gateway_routing::preflight_replacement(
            "gateway-lan",
            "192.0.2.0/24",
            "capture-path",
            &local_subnets,
        )
    };
    verify_replacement().unwrap();
    process::ip(&["rule", "add", "priority", "4199", "lookup", "main"]);
    assert!(verify_replacement().is_err());
    assert!(
        process::ip(&["-j", "route", "show", "table", "420"])
            .as_array()
            .unwrap()
            .iter()
            .any(|route| route["dev"] == "capture-path")
    );
    process::ip(&["rule", "delete", "priority", "4199", "lookup", "main"]);
    verify_replacement().unwrap();
    let destination = "198.51.100.1";
    let forwarded_lookup = [
        "-j",
        "route",
        "get",
        destination,
        "from",
        "192.0.2.10",
        "iif",
        "gateway-lan",
    ];
    assert_eq!(process::ip(&forwarded_lookup)[0]["dev"], "capture-path");
    assert_eq!(
        process::ip(&["-j", "route", "get", destination])[0]["dev"],
        "gateway-lan"
    );
    let capture_index = process::ip(&["-j", "link", "show", "dev", "capture-path"])[0]["ifindex"]
        .as_u64()
        .unwrap();
    let index = |value| NonZeroU32::new(u32::try_from(value).unwrap()).unwrap();
    apply(
        &gateway::capture_transaction(
            index(gateway_index),
            index(capture_index),
            "192.0.2.0".parse().unwrap(),
            24,
        )
        .unwrap(),
    )
    .unwrap();
    assert!(
        !ping_router(),
        "capture path leaked traffic to the physical router"
    );
    let ping_capture = || {
        client
            .command("ping")
            .args(["-c", "1", "-W", "2", "198.51.100.2"])
            .output()
            .unwrap()
            .status
            .success()
    };
    assert!(
        ping_capture(),
        "capture return traffic did not reach the client"
    );
    let directory = tempfile::tempdir().unwrap();
    let ready = directory.path().join("ready");
    let event_path = directory.path().join("events.jsonl");
    let executable = std::env::current_exe().unwrap();
    let test_binary = executable.to_str().unwrap();
    let _echo = ChildGuard(
        capture
            .command(test_binary)
            .args(["--ignored", "--exact", "gateway_echo_worker"])
            .env("XRAT_GATEWAY_ECHO_EVENTS", &event_path)
            .env("XRAT_GATEWAY_ECHO_READY", &ready)
            .spawn()
            .unwrap(),
    );
    process::wait_for(|| ready.exists(), 5, "gateway echo server did not start");
    process::checked(client.command(test_binary).args([
        "--ignored",
        "--exact",
        "gateway_client_worker",
    ]));
    let events = fixtures::Events::file(event_path).all();
    for protocol in ["origin-tcp", "origin-udp"] {
        assert!(events.contains(&serde_json::json!([protocol, "192.0.2.10", CAPTURE_ECHO])));
    }
    for name in ["gateway-udp.test", "gateway-tcp.test"] {
        assert!(events.contains(&serde_json::json!(["dns", name, 1])));
    }
    let rules = process::checked(Command::new("nft").args([
        "-j",
        "-a",
        "list",
        "chain",
        "ip",
        "xrat_gateway",
        "forward",
    ]));
    let rules: serde_json::Value = serde_json::from_slice(&rules.stdout).unwrap();
    let allowance = rules["nftables"]
        .as_array()
        .unwrap()
        .iter()
        .map(|object| &object["rule"])
        .find(|rule| rule["comment"] == gateway::CAPTURE_COMMENT)
        .unwrap();
    assert!(allowance["expr"].as_array().unwrap().iter().any(|expr| {
        expr["counter"]["packets"]
            .as_u64()
            .is_some_and(|packets| packets > 0)
    }));
    let mut close = String::new();
    for comment in [
        gateway::CAPTURE_COMMENT,
        gateway::RETURN_COMMENT,
        gateway::CAPTURE_DROP_COMMENT,
    ] {
        let handle = rules["nftables"]
            .as_array()
            .unwrap()
            .iter()
            .map(|object| &object["rule"])
            .find(|rule| rule["comment"] == comment)
            .unwrap()["handle"]
            .as_u64()
            .unwrap();
        close.push_str(&format!(
            "delete rule ip xrat_gateway forward handle {handle}\n"
        ));
    }
    apply(&close).unwrap();
    assert!(
        !ping_capture(),
        "capture path was still open after closing allowances"
    );
    let remove_routes = || {
        xrat_support::net::gateway_routing::remove_with_local(
            "gateway-lan",
            "192.0.2.0/24",
            "capture-path",
            &local_subnets,
        )
    };
    process::ip(&[
        "route",
        "add",
        "blackhole",
        "203.0.113.0/24",
        "table",
        "420",
    ]);
    assert!(
        remove_routes().is_err(),
        "teardown accepted a foreign table entry"
    );
    assert_eq!(process::ip(&forwarded_lookup)[0]["dev"], "capture-path");
    process::ip(&[
        "route",
        "delete",
        "blackhole",
        "203.0.113.0/24",
        "table",
        "420",
    ]);
    remove_routes().unwrap();
    install_routes().unwrap();
    process::ip(&["link", "delete", "capture-path"]);
    let lookup = Command::new("ip").args(forwarded_lookup).output().unwrap();
    assert!(
        !lookup.status.success(),
        "forwarded lookup fell back to the main table"
    );
    assert_eq!(
        process::ip(&["-j", "route", "get", destination])[0]["dev"],
        "gateway-lan"
    );
    remove_routes().unwrap();
    remove_routes().unwrap();
    lan_forwarding.restore().unwrap();
    process::checked(Command::new("nft").args(["delete", "table", "ip", "xrat_gateway"]));
    assert!(
        !ping_router(),
        "client bypassed the gateway after forwarding stopped"
    );
}
