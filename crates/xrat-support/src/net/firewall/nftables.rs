use std::io::{self, Write};
use std::process::Stdio;

use super::find_in_path;
use super::types::KillSwitchConfig;

pub const TABLE_NAME: &str = "xrat_killswitch";

pub fn is_available() -> bool {
    find_in_path("nft").is_some()
}

pub fn render_ruleset(config: &KillSwitchConfig) -> String {
    let mut rules = String::new();
    rules.push_str(&format!("add table inet {TABLE_NAME}\n"));
    rules.push_str(&format!("flush table inet {TABLE_NAME}\n"));
    rules.push_str(&format!("table inet {TABLE_NAME} {{\n"));
    rules.push_str("    chain output {\n");
    rules.push_str("        type filter hook output priority -10; policy drop;\n");
    rules.push_str("        oifname \"lo\" accept\n");

    let tun = config.tun_interface.trim();
    if !tun.is_empty() {
        rules.push_str(&format!("        oifname \"{tun}\" accept\n"));
    }

    if let Some(mark) = config.socket_mark.filter(|m| *m > 0) {
        rules.push_str(&format!("        meta mark {mark:#x} accept\n"));
    }

    for endpoint in &config.allowed_endpoints {
        match endpoint {
            std::net::SocketAddr::V4(v4) => {
                rules.push_str(&format!(
                    "        ip daddr {} tcp dport {} accept\n",
                    v4.ip(),
                    v4.port()
                ));
                rules.push_str(&format!(
                    "        ip daddr {} udp dport {} accept\n",
                    v4.ip(),
                    v4.port()
                ));
            }
            std::net::SocketAddr::V6(v6) => {
                if !config.block_ipv6 {
                    rules.push_str(&format!(
                        "        ip6 daddr {} tcp dport {} accept\n",
                        v6.ip(),
                        v6.port()
                    ));
                    rules.push_str(&format!(
                        "        ip6 daddr {} udp dport {} accept\n",
                        v6.ip(),
                        v6.port()
                    ));
                }
            }
        }
    }

    rules.push_str("        udp sport 68 udp dport 67 accept\n");
    rules.push_str("        udp sport 546 udp dport 547 accept\n");
    rules.push_str("        ip6 nexthdr icmpv6 icmpv6 type { nd-router-solicit, nd-router-advert, nd-neighbor-solicit, nd-neighbor-advert } accept\n");

    if config.allow_lan {
        rules.push_str("        ip daddr { 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16, 224.0.0.0/4, 255.255.255.255 } accept\n");
        rules.push_str("        ip6 daddr { fe80::/10, fc00::/7, ff00::/8 } accept\n");
    }

    rules.push_str("        counter drop\n");
    rules.push_str("    }\n");
    rules.push_str("}\n");
    rules
}

pub fn apply(config: &KillSwitchConfig) -> io::Result<()> {
    let ruleset = render_ruleset(config);
    let mut child = std::process::Command::new("nft")
        .args(["-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(ruleset.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::other(format!(
            "nft failed to apply kill switch rules: {stderr}"
        )));
    }
    Ok(())
}

pub fn remove() -> io::Result<()> {
    let output = std::process::Command::new("nft")
        .args(["delete", "table", "inet", TABLE_NAME])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("No such file or directory") {
            return Ok(());
        }
        return Err(io::Error::other(format!(
            "nft failed to remove kill switch table: {stderr}"
        )));
    }
    Ok(())
}

pub fn is_active() -> io::Result<bool> {
    let output = std::process::Command::new("nft")
        .args(["list", "table", "inet", TABLE_NAME])
        .output()?;

    if output.status.success() {
        return Ok(true);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("No such file or directory") {
        return Ok(false);
    }
    Err(io::Error::other(format!(
        "nft failed to query kill switch table: {stderr}"
    )))
}
