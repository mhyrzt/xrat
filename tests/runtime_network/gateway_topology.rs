use std::{num::NonZeroU32, process::Command};

use xrat_support::net::firewall::gateway;

use super::{
    fixtures,
    process::{self, ChildGuard},
};

struct Peer {
    worker: ChildGuard,
}

impl Peer {
    fn new(port: &str, address: &str, bridge: Option<&str>) -> Self {
        let worker = ChildGuard(
            Command::new("unshare")
                .args(["--net", "sleep", "infinity"])
                .spawn()
                .unwrap(),
        );
        let peer = Self { worker };
        let pid = peer.worker.0.id().to_string();
        let gateway_namespace = std::fs::read_link("/proc/self/ns/net").unwrap();
        process::wait_for(
            || {
                std::fs::read_link(format!("/proc/{pid}/ns/net"))
                    .is_ok_and(|namespace| namespace != gateway_namespace)
            },
            5,
            "peer network namespace was not created",
        );
        process::ip(&[
            "link", "add", port, "type", "veth", "peer", "name", "lan", "netns", &pid,
        ]);
        if let Some(bridge) = bridge {
            process::ip(&["link", "set", port, "master", bridge]);
        }
        process::ip(&["link", "set", port, "up"]);
        peer.ip(&["link", "set", "lo", "up"]);
        peer.ip(&["address", "add", address, "dev", "lan"]);
        peer.ip(&["link", "set", "lan", "up"]);
        peer
    }

    fn command(&self, program: &str) -> Command {
        let mut command = Command::new("nsenter");
        command.args([
            "--target",
            &self.worker.0.id().to_string(),
            "--net",
            program,
        ]);
        command
    }

    fn ip(&self, arguments: &[&str]) {
        process::checked(self.command("ip").args(arguments));
    }
}

const CAPTURE_ECHO: &str = "198.51.100.2";

pub fn echo_worker() {
    process::assert_isolated();
    let events =
        fixtures::Events::file(std::env::var_os("XRAT_GATEWAY_ECHO_EVENTS").unwrap().into());
    if std::env::var_os("XRAT_GATEWAY_SOCKS_SERVER").is_some() {
        fixtures::start(events.clone(), &[CAPTURE_ECHO], "::1", true);
        fixtures::start_socks("192.0.2.2:1082".parse().unwrap(), &events);
    } else {
        fixtures::start_echo(CAPTURE_ECHO.parse().unwrap(), &events, true);
    }
    fixtures::start_dns(
        std::net::SocketAddr::new(CAPTURE_ECHO.parse().unwrap(), 53),
        &events,
        "::1",
    );
    std::fs::write(std::env::var_os("XRAT_GATEWAY_ECHO_READY").unwrap(), "").unwrap();
    loop {
        std::thread::park();
    }
}

pub fn client_worker() {
    process::assert_isolated();
    fixtures::send(CAPTURE_ECHO, false);
    fixtures::send(CAPTURE_ECHO, true);
    for (name, tcp) in [("gateway-udp.test", false), ("gateway-tcp.test", true)] {
        assert_eq!(
            fixtures::lookup(name, tcp, 1, CAPTURE_ECHO, 53).unwrap(),
            (0, Some("203.0.113.2".parse().unwrap()))
        );
    }
}

pub fn closed_client_worker() {
    use std::{
        io,
        net::{SocketAddr, TcpStream, UdpSocket},
        time::Duration,
    };

    process::assert_isolated();
    let destination = CAPTURE_ECHO.parse().unwrap();
    let timeout = Duration::from_millis(600);
    assert!(
        TcpStream::connect_timeout(&SocketAddr::new(destination, 8080), timeout).is_err(),
        "client reached the TCP origin while the gateway should be closed"
    );
    let udp = UdpSocket::bind("0.0.0.0:0").unwrap();
    udp.connect(SocketAddr::new(destination, 8081)).unwrap();
    udp.set_read_timeout(Some(timeout)).unwrap();
    if udp.send(b"xrat").is_ok() {
        let error = udp
            .recv(&mut [0; 8])
            .expect_err("UDP origin bypassed the closed gateway");
        assert!(matches!(
            error.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::ConnectionRefused
        ));
    }
}

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

pub fn exercise_managed_engines() {
    use super::tun::{Cli, core_pid};
    use std::{fs, path::PathBuf};

    process::assert_isolated();
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/net/tun")
        .expect("managed gateway acceptance requires an accessible /dev/net/tun");
    let binary = std::env::var_os("XRAT_RUNTIME_CLI")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_xrat")));
    let binary = fs::canonicalize(binary).unwrap();
    let xray = process::path("XRAT_RUNTIME_XRAY");
    let singbox = process::path("XRAT_RUNTIME_SINGBOX");
    let output = process::directory("XRAT_RUNTIME_OUTPUT");
    let executable = std::env::current_exe().unwrap();
    let test_binary = executable.to_str().unwrap();

    process::ip(&["link", "set", "lo", "up"]);
    process::ip(&["link", "add", "gateway-lan", "type", "bridge"]);
    process::ip(&["address", "add", "192.0.2.1/24", "dev", "gateway-lan"]);
    process::ip(&["link", "set", "gateway-lan", "up"]);
    for device in ["all", "default", "gateway-lan"] {
        for key in ["forwarding", "send_redirects", "rp_filter"] {
            fs::write(format!("/proc/sys/net/ipv4/conf/{device}/{key}"), "0").unwrap();
        }
    }
    let router = Peer::new("router-port", "192.0.2.2/24", Some("gateway-lan"));
    router.ip(&["address", "add", "198.51.100.2/32", "dev", "lo"]);
    let client = Peer::new("client-port", "192.0.2.10/24", Some("gateway-lan"));
    client.ip(&["route", "add", "default", "via", "192.0.2.1"]);
    process::ip(&["route", "add", "default", "via", "192.0.2.2"]);

    let evidence = tempfile::tempdir_in(&output).unwrap().keep();
    let ready = evidence.join("ready");
    let event_path = evidence.join("events.jsonl");
    let _proxy = ChildGuard(
        router
            .command(test_binary)
            .args(["--ignored", "--exact", "gateway_echo_worker"])
            .env("XRAT_GATEWAY_ECHO_EVENTS", &event_path)
            .env("XRAT_GATEWAY_ECHO_READY", &ready)
            .env("XRAT_GATEWAY_SOCKS_SERVER", "1")
            .spawn()
            .unwrap(),
    );
    process::wait_for(|| ready.exists(), 5, "upstream proxy did not start");
    let events = fixtures::Events::file(event_path);
    let assert_closed_client = || {
        process::checked(client.command(test_binary).args([
            "--ignored",
            "--exact",
            "gateway_closed_client_worker",
        ]));
    };
    let status = |cli: &Cli| -> serde_json::Value {
        serde_json::from_slice(&cli.run(&["gateway", "status", "--json"]).stdout).unwrap()
    };

    for engine in ["xray", "sing-box"] {
        let root = evidence.join(engine);
        fs::create_dir(&root).unwrap();
        fs::write(
            root.join("config.toml"),
            format!(
                r#"
[runtime]
engine = "{engine}"
[runtime.tun]
enabled = true
interface_name = "xrat-gateway"
auto_route = false
address = ["172.19.0.1/30"]
[gateway]
interface = "gateway-lan"
lan_subnet = "192.0.2.0/24"
"#
            ),
        )
        .unwrap();
        let cli = Cli {
            root: root.clone(),
            binary: binary.clone(),
            xray: xray.clone(),
            singbox: singbox.clone(),
        };
        cli.run(&[
            "import",
            "socks5://192.0.2.2:1081",
            "--name",
            "gateway-fixture",
        ]);
        let records: serde_json::Value =
            serde_json::from_slice(&cli.run(&["list", "configs", "--format", "json"]).stdout)
                .unwrap();
        let identifier = records[0]["ref"].as_str().unwrap();
        cli.run(&[
            "import",
            "socks5://192.0.2.2:1082",
            "--name",
            "gateway-alternate",
        ]);
        let alternatives: serde_json::Value =
            serde_json::from_slice(&cli.run(&["list", "configs", "--format", "json"]).stdout)
                .unwrap();
        let alternate = alternatives
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|entry| entry["ref"].as_str())
            .find(|reference| *reference != identifier)
            .expect("alternate gateway config was not imported");
        cli.run(&["gateway", "install", "--dry-run"]);
        cli.run(&["gateway", "install"]);
        assert_eq!(status(&cli)["protection"], "closed_guard_observed");
        assert_closed_client();
        let mut daemon = cli.foreground();
        let start = events.all().len();
        cli.run(&["connect", identifier, "--json"]);
        assert_eq!(status(&cli)["protection"], "active_guard_observed");
        process::checked(client.command(test_binary).args([
            "--ignored",
            "--exact",
            "gateway_client_worker",
        ]));
        let receipts = events.all();
        for (event, port) in [("proxy", 8080), ("proxy-udp", 8081)] {
            assert!(receipts[start..].contains(&serde_json::json!([event, CAPTURE_ECHO, port])));
        }
        for event in &receipts[start..] {
            if matches!(event[0].as_str(), Some("origin-tcp" | "origin-udp")) {
                assert_ne!(event[1], "192.0.2.10", "client bypassed the proxy");
            }
        }
        let original_pid = core_pid(&root);
        cli.run(&["rotate", "now", "--config-id", alternate]);
        assert_ne!(original_pid, core_pid(&root));
        assert_eq!(status(&cli)["protection"], "active_guard_observed");
        process::checked(client.command(test_binary).args([
            "--ignored",
            "--exact",
            "gateway_client_worker",
        ]));
        process::checked(Command::new("kill").args(["-KILL", &core_pid(&root).to_string()]));
        assert_closed_client();
        cli.run(&["disconnect"]);
        assert_eq!(status(&cli)["protection"], "closed_guard_observed");
        cli.run(&["connect", identifier, "--json"]);
        process::checked(client.command(test_binary).args([
            "--ignored",
            "--exact",
            "gateway_client_worker",
        ]));
        cli.run(&["disconnect"]);
        assert_closed_client();
        assert_eq!(status(&cli)["protection"], "closed_guard_observed");
        daemon.stop();
        cli.run(&["gateway", "remove", "--dry-run"]);
        cli.run(&["gateway", "remove"]);
        assert_eq!(status(&cli)["installation"], "absent");
    }
}
