use super::{
    fixtures::{self, Events},
    process::{self, ChildGuard},
};
use serde_json::{Value, json};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufRead, Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    thread,
    time::{Duration, Instant},
};

pub fn serve() {
    let directory = process::directory("XRAT_RUNTIME_OUTPUT");
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).unwrap();
    assert_eq!(line.trim(), "start");
    process::ip(&["link", "set", "lo", "up"]);
    process::ip(&["link", "set", "peer", "up"]);
    for address in ["192.0.2.2/24", "2001:db8:1::2/64"] {
        process::ip(&["address", "add", address, "dev", "peer"]);
    }
    for address in ["203.0.113.2/32", "198.51.100.2/32", "2001:db8:2::2/128"] {
        process::ip(&["address", "add", address, "dev", "lo"]);
    }
    fixtures::start(
        Events::file(directory.join("events.jsonl")),
        &["203.0.113.2", "198.51.100.2", "2001:db8:2::2"],
        "2001:db8:2::2",
        true,
    );
    fs::write(directory.join("server-ready"), "ready").unwrap();
    io::stdin().read_to_end(&mut Vec::new()).unwrap();
}
fn snapshot() -> Value {
    json!({"addresses": process::ip(&["-j", "address"]), "routes4": process::ip(&["-j", "route", "show", "table", "all"]), "routes6": process::ip(&["-6", "-j", "route", "show", "table", "all"]), "rules4": process::ip(&["-j", "rule"]), "rules6": process::ip(&["-6", "-j", "rule"])})
}
fn restored(baseline: &Value, seconds: u64, message: &str) {
    process::wait_for(|| snapshot() == *baseline, seconds, message);
}
fn has_tun() -> bool {
    snapshot()["addresses"]
        .as_array()
        .unwrap()
        .iter()
        .any(|link| link["ifname"] == "xrat-proof")
}
fn dns(name: &str, tcp: bool, host: &str, port: u16) {
    assert_eq!(
        fixtures::lookup(name, tcp, 1, host, port)
            .unwrap()
            .1
            .unwrap()
            .to_string(),
        "203.0.113.2"
    );
}
fn proxy() {
    fixtures::recover_fake("203.0.113.2".parse().unwrap());
}

pub(super) struct Cli {
    pub(super) root: PathBuf,
    pub(super) binary: PathBuf,
    pub(super) xray: PathBuf,
    pub(super) singbox: PathBuf,
}
impl Cli {
    fn command(&self) -> Command {
        let mut command = Command::new(&self.binary);
        command
            .arg("--config")
            .arg(self.root.join("config.toml"))
            .arg("--database")
            .arg(self.root.join("db.sqlite"))
            .arg("--xray")
            .arg(&self.xray)
            .arg("--sing-box")
            .arg(&self.singbox)
            .env("XRAT_PATH", &self.root)
            .env("XDG_RUNTIME_DIR", &self.root)
            .env("NO_COLOR", "1");
        command
    }
    fn call(&self, action: &[&str], check: bool) -> Output {
        let deadline = Instant::now() + Duration::from_secs(15);
        let result = loop {
            let stdout = tempfile::tempfile().unwrap();
            let stderr = tempfile::tempfile().unwrap();
            let mut child = ChildGuard(
                self.command()
                    .args(action)
                    .stdout(stdout.try_clone().unwrap())
                    .stderr(stderr.try_clone().unwrap())
                    .spawn()
                    .unwrap(),
            );
            child.wait(30);
            let mut stdout = stdout;
            let mut stderr = stderr;
            use std::io::{Seek, SeekFrom};
            stdout.seek(SeekFrom::Start(0)).unwrap();
            stderr.seek(SeekFrom::Start(0)).unwrap();
            let mut out = Vec::new();
            let mut err = Vec::new();
            stdout.read_to_end(&mut out).unwrap();
            stderr.read_to_end(&mut err).unwrap();
            let result = Output {
                status: child.0.wait().unwrap(),
                stdout: out,
                stderr: err,
            };
            if !String::from_utf8_lossy(&result.stderr).contains("daemon ping timed out")
                || Instant::now() >= deadline
            {
                break result;
            }
            thread::sleep(Duration::from_millis(200));
        };
        writeln!(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.root.join("commands.log"))
                .unwrap(),
            "{}\n{}{}",
            json!(action),
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        )
        .unwrap();
        assert!(
            !check || result.status.success(),
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        result
    }
    pub(super) fn run(&self, action: &[&str]) -> Output {
        self.call(action, true)
    }
    pub(super) fn foreground(&self) -> ChildGuard {
        let log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.root.join("foreground-daemon.log"))
            .unwrap();
        let child = ChildGuard(
            self.command()
                .args(["daemon", "run-server"])
                .stdout(log.try_clone().unwrap())
                .stderr(log)
                .spawn()
                .unwrap(),
        );
        process::wait_for(
            || self.root.join("runtime/daemon.sock").exists(),
            10,
            "foreground daemon not ready",
        );
        child
    }
}
impl Drop for Cli {
    fn drop(&mut self) {
        self.call(&["disconnect"], false);
        self.call(&["daemon", "stop"], false);
    }
}
fn config(xray: &Path, singbox: &Path, engine: &str, stack: &str, ipv6: bool) -> String {
    let mut addresses = vec!["172.19.0.1/30"];
    if ipv6 {
        addresses.push("fdfe::1/126");
    }
    let excluded = if engine == "sing-box" {
        vec!["198.51.100.2/32"]
    } else {
        vec![]
    };
    format!(
        r#"[paths]
xray = {xray}
sing_box = {singbox}
[runtime]
engine = "{engine}"
[runtime.socks]
port = 1080
[runtime.stats]
enabled = false
[runtime.tun]
enabled = true
interface_name = "xrat-proof"
mtu = 1400
stack = "{stack}"
address = {addresses}
route_exclude_address = {excluded}
[dns]
query_strategy = "UseIP"
use_system_hosts = false
bootstrap_resolver = "bootstrap"
final_resolver = "remote"
resolvers = [{{tag="bootstrap",address="udp://192.0.2.2:5353",path="bootstrap"}},{{tag="remote",address="tcp://192.0.2.2:5353",path="proxy"}}]
[dns.listener]
enabled = true
port = 1053
"#,
        xray = json!(xray),
        singbox = json!(singbox),
        addresses = json!(addresses),
        excluded = json!(excluded)
    )
}
pub(super) fn core_pid(root: &Path) -> i64 {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        use sqlx::Connection;
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(root.join("db.sqlite"))
            .read_only(true);
        let mut database = sqlx::SqliteConnection::connect_with(&options)
            .await
            .unwrap();
        sqlx::query_scalar("select process_id from runtime_sessions order by id desc limit 1")
            .fetch_one(&mut database)
            .await
            .unwrap()
    })
}

fn split_traffic(cli: &Cli, events: &Events) -> Value {
    let mut receipt = Vec::new();
    let application_dir = cli.root.join("split/helper-app");
    fs::create_dir_all(&application_dir).unwrap();
    let applications = [
        cli.root.join("split/name-client"),
        cli.root.join("split/path-client"),
        application_dir.join("worker"),
        cli.root.join("split/other-client"),
    ];
    for executable in &applications {
        fs::copy(std::env::current_exe().unwrap(), executable).unwrap();
    }
    for list in ["blacklist", "whitelist"] {
        cli.run(&[
            "tun",
            "split",
            "add",
            list,
            "name-client",
            applications[1].to_str().unwrap(),
            &format!("{}/", application_dir.display()),
        ]);
    }
    for mode in ["whitelist", "blacklist"] {
        cli.run(&["tun", "split", "mode", mode]);
        for (index, executable) in applications.iter().enumerate() {
            let start = events.all().len();
            process::checked(Command::new(executable).args([
                "--ignored",
                "--exact",
                "split_traffic_worker",
                "--nocapture",
            ]));
            let proxied = events.all()[start..].contains(&json!(["proxy", "203.0.113.2", 8080]));
            assert_eq!(
                proxied,
                (index < 3) == (mode == "whitelist"),
                "{mode}: wrong route for {}",
                executable.display()
            );
            receipt.push(json!({"mode": mode, "executable": executable, "proxied": proxied}));
        }
        dns("split-protected-dns.test", false, "203.0.113.53", 53);
    }
    cli.run(&["tun", "split", "mode", "all"]);
    cli.run(&["tun", "split", "clear", "both"]);
    json!(receipt)
}
pub fn exercise() {
    let directory = process::directory("XRAT_RUNTIME_OUTPUT");
    let xray = process::path("XRAT_RUNTIME_XRAY");
    let singbox = process::path("XRAT_RUNTIME_SINGBOX");
    let binary = std::env::var_os("XRAT_RUNTIME_CLI")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_xrat")));
    let binary = fs::canonicalize(binary).unwrap();
    let ready = directory.join("server-ready");
    let _ = fs::remove_file(&ready);
    fs::write(directory.join("events.jsonl"), "").unwrap();
    process::ip(&["link", "set", "lo", "up"]);
    let log = File::create(directory.join("fixture-server.log")).unwrap();
    let mut server = ChildGuard(
        Command::new("unshare")
            .arg("--net")
            .arg(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "fixture_worker", "--nocapture"])
            .stdin(Stdio::piped())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .unwrap(),
    );
    process::ip(&[
        "link", "add", "physical", "type", "veth", "peer", "name", "peer",
    ]);
    process::ip(&["link", "set", "peer", "netns", &server.0.id().to_string()]);
    process::ip(&["link", "set", "physical", "up"]);
    for address in ["192.0.2.1/24", "2001:db8:1::1/64"] {
        process::ip(&["address", "add", address, "dev", "physical"]);
    }
    server
        .0
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"start\n")
        .unwrap();
    process::wait_for(|| ready.exists(), 10, "fixture server not ready");
    process::ip(&["route", "add", "default", "via", "192.0.2.2"]);
    process::ip(&["-6", "route", "add", "default", "via", "2001:db8:1::2"]);
    process::ip(&["link", "add", "foreign0", "type", "dummy"]);
    process::ip(&["link", "set", "foreign0", "up"]);
    process::ip(&["address", "add", "10.250.0.1/24", "dev", "foreign0"]);
    thread::sleep(Duration::from_secs(2));
    let baseline = snapshot();
    process::write_json(&directory.join("baseline-network.json"), &baseline);
    let events = Events::file(directory.join("events.jsonl"));
    let mut receipts = Vec::new();
    for (number, (engine, stack, ipv6)) in [
        ("xray", "system", false),
        ("xray", "system", true),
        ("sing-box", "system", false),
        ("sing-box", "gvisor", false),
        ("sing-box", "mixed", true),
    ]
    .into_iter()
    .enumerate()
    {
        let root = directory.join(format!("{number}-{engine}-{stack}"));
        fs::create_dir_all(&root).unwrap();
        let configuration = root.join("config.toml");
        fs::write(&configuration, config(&xray, &singbox, engine, stack, ipv6)).unwrap();
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
            "namespace-fixture",
        ]);
        let records: Value =
            serde_json::from_slice(&cli.run(&["list", "configs", "--format", "json"]).stdout)
                .unwrap();
        let identifier = records[0]["ref"].as_str().unwrap();
        cli.run(&["daemon", "start"]);
        cli.run(&["connect", identifier, "--json"]);
        process::wait_for(has_tun, 10, "TUN not created");
        let active = snapshot();
        process::write_json(&root.join("active-network.json"), &active);
        assert_eq!(
            active["addresses"]
                .as_array()
                .unwrap()
                .iter()
                .find(|link| link["ifname"] == "xrat-proof")
                .unwrap()["mtu"],
            1400
        );
        let start = events.all().len();
        fixtures::send("203.0.113.2", false);
        fixtures::send("203.0.113.2", true);
        let captured = events.all();
        assert!(
            captured[start..].contains(&json!(["proxy", "203.0.113.2", 8080])),
            "TCP was not captured"
        );
        assert!(
            captured[start..].contains(&json!(["proxy-udp", "203.0.113.2", 8081])),
            "UDP was not captured"
        );
        if ipv6 {
            fixtures::send("2001:db8:2::2", false);
            fixtures::send("2001:db8:2::2", true);
        }
        proxy();
        dns("remote.test", false, "127.0.0.1", 1053);
        dns("captured-udp.test", false, "203.0.113.53", 53);
        dns("captured-tcp.test", true, "203.0.113.53", 53);
        if ipv6 {
            dns("captured-udp6.test", false, "2001:db8:2::53", 53);
            dns("captured-tcp6.test", true, "2001:db8:2::53", 53);
        }
        if engine == "sing-box" {
            fixtures::send("198.51.100.2", false);
            fixtures::send("198.51.100.2", true);
        }
        let traffic = events.all()[start..].to_vec();
        if ipv6 {
            assert!(traffic.contains(&json!(["proxy", "2001:db8:2::2", 8080])));
            assert!(traffic.contains(&json!(["proxy-udp", "2001:db8:2::2", 8081])));
        }
        if engine == "sing-box" {
            assert!(
                traffic.contains(&json!(["origin-tcp", "192.0.2.1", "198.51.100.2"])),
                "excluded LAN TCP was captured"
            );
            assert!(
                traffic.contains(&json!(["origin-udp", "192.0.2.1", "198.51.100.2"])),
                "excluded LAN UDP was captured"
            );
        }
        let split_receipt = split_traffic(&cli, &events);
        cli.run(&["tun", "disable", "--json"]);
        restored(&baseline, 10, "live disable did not restore proxy mode");
        proxy();
        let healthy = fs::read_to_string(&configuration).unwrap();
        fs::write(
            &configuration,
            healthy
                .replace("udp://192.0.2.2:5353", "udp://192.0.2.254:5353")
                .replace("tcp://192.0.2.2:5353", "tcp://resolver.test:5353"),
        )
        .unwrap();
        let rejection = cli.call(&["tun", "enable", "--json"], false);
        assert!(
            !rejection.status.success()
                && String::from_utf8_lossy(&rejection.stderr)
                    .to_lowercase()
                    .contains("bootstrap"),
            "failed bootstrap was accepted"
        );
        assert_eq!(snapshot(), baseline);
        proxy();
        fs::write(&configuration, &healthy).unwrap();
        if engine == "xray" {
            let unknown = root.join("unknown-xray");
            fs::write(&unknown, "#!/bin/sh\nprintf 'Xray unknown\\n'\n").unwrap();
            fs::set_permissions(&unknown, fs::Permissions::from_mode(0o755)).unwrap();
            for unsupported in [xray.with_file_name("xray-old"), unknown] {
                fs::write(
                    &configuration,
                    healthy.replace(&json!(xray).to_string(), &json!(unsupported).to_string()),
                )
                .unwrap();
                let rejection = cli.call(&["tun", "enable", "--json"], false);
                assert!(
                    !rejection.status.success()
                        && String::from_utf8_lossy(&rejection.stderr).contains("26.7.11"),
                    "unsupported Xray version was accepted"
                );
                assert_eq!(snapshot(), baseline);
                proxy();
            }
            fs::write(&configuration, &healthy).unwrap();
        }
        process::ip(&["link", "add", "xrat-proof", "type", "dummy"]);
        let collision = snapshot();
        assert!(
            !cli.call(&["tun", "enable", "--json"], false)
                .status
                .success(),
            "foreign interface was accepted"
        );
        assert_eq!(snapshot(), collision);
        proxy();
        process::ip(&["link", "delete", "xrat-proof"]);
        if engine == "sing-box" {
            process::ip(&["rule", "add", "priority", "9000", "table", "main"]);
            let collision = snapshot();
            assert!(
                !cli.call(&["tun", "enable", "--json"], false)
                    .status
                    .success(),
                "foreign capture priority was accepted"
            );
            assert_eq!(snapshot(), collision);
            proxy();
            process::ip(&["rule", "delete", "priority", "9000", "table", "main"]);
        }
        cli.run(&["tun", "enable", "--json"]);
        fixtures::send("203.0.113.2", false);
        fixtures::send("203.0.113.2", true);
        cli.run(&["disconnect"]);
        process::write_json(&root.join("after-disconnect-network.json"), &snapshot());
        restored(&baseline, 10, "disconnect did not restore namespace state");
        cli.run(&["connect", identifier, "--json"]);
        process::checked(Command::new("kill").args(["-KILL", &core_pid(&root).to_string()]));
        restored(&baseline, 35, "core crash did not restore namespace state");
        cli.run(&["connect", identifier, "--json"]);
        cli.run(&["daemon", "stop"]);
        restored(&baseline, 10, "daemon shutdown left network state");
        process::wait_for(
            || !root.join("runtime/daemon.sock").exists(),
            10,
            "old daemon socket not removed",
        );
        let mut owner = cli.foreground();
        cli.run(&["connect", identifier, "--json"]);
        owner.signal("-KILL");
        owner.wait(5);
        assert!(has_tun(), "daemon crash lost live core");
        owner = cli.foreground();
        process::wait_for(
            || {
                String::from_utf8_lossy(&cli.call(&["daemon", "status"], false).stdout)
                    .contains("runtime status available")
            },
            10,
            "daemon did not reattach",
        );
        proxy();
        fixtures::send("203.0.113.2", false);
        fixtures::send("203.0.113.2", true);
        owner.signal("-INT");
        owner.wait(30);
        restored(&baseline, 30, "SIGINT left network state");
        owner = cli.foreground();
        cli.run(&["connect", identifier, "--json"]);
        owner.signal("-TERM");
        owner.wait(30);
        restored(&baseline, 30, "SIGTERM left network state");
        receipts.push(json!({"engine": engine, "stack": stack, "ipv6": ipv6, "events": traffic, "split": split_receipt, "before": baseline, "active": active, "after": snapshot()}));
        println!("PASS {engine}/{stack}/ipv6={ipv6}");
    }
    assert_eq!(receipts.len(), 5);
    process::write_json(
        &directory.join("tun-network-receipts.json"),
        &json!(receipts),
    );
    server.stop();
}
