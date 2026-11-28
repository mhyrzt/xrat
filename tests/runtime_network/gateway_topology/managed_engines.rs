use super::*;

pub fn exercise_managed_engines() {
    use super::super::tun::{Cli, core_pid};
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
        let apply = xrat_support::net::firewall::nftables::apply_transaction;
        apply("create table ip xrat_gateway { comment \"foreign fixture\"; }\n").unwrap();
        for action in [
            &["gateway", "install", "--dry-run"][..],
            &["gateway", "install"][..],
        ] {
            let rejected = cli.call(action, false);
            assert!(!rejected.status.success());
            assert!(String::from_utf8_lossy(&rejected.stderr).contains("different owner or shape"));
            assert_eq!(status(&cli)["installation"], "absent");
        }
        apply("delete table ip xrat_gateway\n").unwrap();
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
        process::ip(&[
            "rule",
            "add",
            "priority",
            "4199",
            "to",
            "127.0.0.0/8",
            "lookup",
            "main",
        ]);
        let rejected = cli.call(&["rotate", "now", "--config-id", alternate], false);
        assert!(!rejected.status.success());
        assert!(
            String::from_utf8_lossy(&rejected.stderr).contains("gateway route preflight failed")
        );
        assert_eq!(original_pid, core_pid(&root));
        assert_eq!(status(&cli)["protection"], "active_guard_observed");
        process::checked(client.command(test_binary).args([
            "--ignored",
            "--exact",
            "gateway_client_worker",
        ]));
        process::ip(&[
            "rule",
            "delete",
            "priority",
            "4199",
            "to",
            "127.0.0.0/8",
            "lookup",
            "main",
        ]);
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
        let config_path = root.join("config.toml");
        let valid_config = fs::read(&config_path).unwrap();
        fs::write(&config_path, "[gateway\n").unwrap();
        cli.run(&["gateway", "remove", "--dry-run"]);
        cli.run(&["gateway", "remove"]);
        fs::write(config_path, valid_config).unwrap();
        assert_eq!(status(&cli)["installation"], "absent");
    }
}
