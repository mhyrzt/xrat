use std::{fs, process::Command};

use serde_json::Value;

use super::process;

fn snapshot() -> Vec<Value> {
    [
        vec!["-j", "address", "show"],
        vec!["-j", "-4", "route", "show", "table", "all"],
        vec!["-j", "-4", "rule", "show"],
    ]
    .iter()
    .map(|arguments| process::ip(arguments))
    .chain(std::iter::once(
        serde_json::from_slice(
            &process::checked(Command::new("nft").args(["-j", "list", "ruleset"])).stdout,
        )
        .unwrap(),
    ))
    .chain(
        [
            "/proc/sys/net/ipv4/ip_forward",
            "/proc/sys/net/ipv4/conf/all/rp_filter",
            "/proc/sys/net/ipv4/conf/all/src_valid_mark",
            "/proc/sys/net/ipv4/conf/xrat-test/rp_filter",
            "/proc/sys/net/ipv4/conf/xrat-test/src_valid_mark",
        ]
        .into_iter()
        .map(|path| Value::String(fs::read_to_string(path).unwrap())),
    )
    .collect()
}

pub fn exercise() {
    process::assert_isolated();
    process::ip(&["link", "add", "xrat-test", "type", "dummy"]);
    process::ip(&["link", "add", "xrat0", "type", "dummy"]);
    process::ip(&["address", "add", "192.0.2.2/24", "dev", "xrat-test"]);
    process::ip(&["link", "set", "xrat-test", "up"]);
    process::ip(&[
        "route",
        "add",
        "default",
        "via",
        "192.0.2.1",
        "dev",
        "xrat-test",
    ]);
    process::ip(&[
        "rule", "add", "priority", "4200", "fwmark", "0x42", "lookup", "123",
    ]);
    process::checked(Command::new("nft").args(["add", "table", "inet", "xrat_fixture"]));
    process::checked(Command::new("nft").arg("add chain inet xrat_fixture forward { type filter hook forward priority 0; policy accept; }"));

    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let database = dir.path().join("db.sqlite");
    let unused = dir.path().join("unused");
    fs::write(&config, "[gateway]\ninterface = 'xrat-test'\n").unwrap();
    let before = snapshot();
    let command = |action: &[&str]| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_xrat"));
        command
            .env("XRAT_PATH", &unused)
            .arg("--config")
            .arg(&config)
            .arg("--database")
            .arg(&database)
            .arg("gateway")
            .args(action);
        command.output().unwrap()
    };
    let doctor = command(&["doctor", "--json"]);
    assert!(doctor.status.success());
    let result: Value = serde_json::from_slice(&doctor.stdout).unwrap();
    let checks = result["preflight"]["checks"].as_array().unwrap();
    let detail = |name: &str| {
        checks.iter().find(|check| check["name"] == name).unwrap()["detail"]
            .as_str()
            .unwrap()
    };
    assert!(detail("rule_marks").contains("4200"));
    assert!(
        detail("firewall_chains").contains("inet/xrat_fixture/forward hook forward priority 0")
    );
    assert!(detail("tun_collision").contains("exists without matching host TUN ownership"));
    assert_eq!(result["preflight"]["ready"], false);
    assert!(
        !command(&["install", "--dry-run", "--json"])
            .status
            .success()
    );
    assert_eq!(before, snapshot());
    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        "[gateway]\ninterface = 'xrat-test'\n"
    );
    assert!(!database.exists());
    fs::write(&config, "[gateway]\nlan_subnet = 'invalid'\n").unwrap();
    assert!(command(&["remove", "--dry-run", "--json"]).status.success());
    fs::write(&config, "[gateway\n").unwrap();
    assert!(command(&["remove", "--dry-run", "--json"]).status.success());
    assert!(!database.exists());
    assert!(!unused.exists());
}
