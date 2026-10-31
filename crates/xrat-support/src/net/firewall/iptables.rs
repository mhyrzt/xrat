use std::io;

use super::find_in_path;
use super::types::KillSwitchConfig;

pub const CHAIN_NAME: &str = "XRAT_KS";
pub const CHAIN_STAGING: &str = "XRAT_KS_STG";

#[cfg(test)]
#[path = "iptables_tests.rs"]
mod tests;

pub fn run_iptables(binary: &str, args: &[&str]) -> io::Result<()> {
    let output = std::process::Command::new(binary).args(args).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("{binary} exited with non-zero status: {}", output.status)
        } else {
            format!("{binary} failed (args: {args:?}): {stderr}")
        };
        return Err(io::Error::other(message));
    }
    Ok(())
}

pub fn check_chain_in_output(binary: &str, chain: &str) -> io::Result<bool> {
    if !chain_exists(binary, chain)? {
        return Ok(false);
    }
    query_iptables(binary, &["-C", "OUTPUT", "-j", chain])
}

fn query_iptables(binary: &str, args: &[&str]) -> io::Result<bool> {
    let output = std::process::Command::new(binary).args(args).output()?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Err(io::Error::other(format!(
            "{binary} query failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))),
    }
}

fn has_active_chain(binary: &str) -> io::Result<bool> {
    Ok(check_chain_in_output(binary, CHAIN_NAME)? || check_chain_in_output(binary, CHAIN_STAGING)?)
}

fn chain_exists(binary: &str, chain: &str) -> io::Result<bool> {
    let output = std::process::Command::new(binary).arg("-S").output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "{binary} failed to list chains: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).lines().any(|line| {
        let mut fields = line.split_whitespace();
        fields.next() == Some("-N") && fields.next() == Some(chain)
    }))
}

pub fn is_ipv6_disabled() -> bool {
    is_ipv6_disabled_at(
        std::path::Path::new("/proc/sys/net/ipv6/conf/all/disable_ipv6"),
        std::path::Path::new("/proc/net/if_inet6"),
    )
}

pub(crate) fn is_ipv6_disabled_at(
    sysctl_path: &std::path::Path,
    if_inet6_path: &std::path::Path,
) -> bool {
    if let Ok(content) = std::fs::read_to_string(sysctl_path)
        && content.trim() == "1"
    {
        return true;
    }
    match std::fs::read_to_string(if_inet6_path) {
        Ok(content) => content.trim().is_empty(),
        Err(_) => true,
    }
}

pub fn is_available() -> bool {
    is_available_internal(
        find_in_path("iptables").is_some(),
        find_in_path("ip6tables").is_some(),
        is_ipv6_disabled(),
    )
}

pub(crate) fn is_available_internal(
    has_iptables: bool,
    has_ip6tables: bool,
    ipv6_disabled: bool,
) -> bool {
    if !has_iptables {
        return false;
    }
    if !ipv6_disabled && !has_ip6tables {
        return false;
    }
    true
}

pub fn is_active() -> io::Result<bool> {
    let v4_active = has_active_chain("iptables")?;
    if !v4_active {
        return Ok(false);
    }
    if !is_ipv6_disabled() {
        if find_in_path("ip6tables").is_none() {
            return Ok(false);
        }
        let v6_active = has_active_chain("ip6tables")?;
        if !v6_active {
            return Ok(false);
        }
    }
    Ok(true)
}

pub fn apply(config: &KillSwitchConfig) -> io::Result<()> {
    apply_v4(config)?;
    apply_v6(config)?;
    Ok(())
}

fn apply_v4(config: &KillSwitchConfig) -> io::Result<()> {
    apply_staging("iptables", config, true)
}

fn apply_v6(config: &KillSwitchConfig) -> io::Result<()> {
    let has_ip6tables = find_in_path("ip6tables").is_some();
    let ipv6_disabled = is_ipv6_disabled();
    apply_v6_staging(config, has_ip6tables, ipv6_disabled)
}

pub(crate) fn apply_v6_staging(
    config: &KillSwitchConfig,
    has_ip6tables: bool,
    ipv6_disabled: bool,
) -> io::Result<()> {
    if !has_ip6tables {
        if ipv6_disabled {
            return Ok(());
        }
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "ip6tables is not available and IPv6 is enabled; cannot establish fail-closed kill switch",
        ));
    }
    apply_staging("ip6tables", config, false)
}

fn cleanup_chain(binary: &str, chain: &str) -> io::Result<()> {
    while check_chain_in_output(binary, chain)? {
        run_iptables(binary, &["-D", "OUTPUT", "-j", chain])?;
    }
    if chain_exists(binary, chain)? {
        run_iptables(binary, &["-F", chain])?;
        run_iptables(binary, &["-X", chain])?;
    }
    Ok(())
}

fn apply_staging(binary: &str, config: &KillSwitchConfig, is_v4: bool) -> io::Result<()> {
    // An interrupted swap may leave staging as the only attached policy.
    if check_chain_in_output(binary, CHAIN_STAGING)? {
        cleanup_chain(binary, CHAIN_NAME)?;
        run_iptables(binary, &["-E", CHAIN_STAGING, CHAIN_NAME])?;
    }
    cleanup_chain(binary, CHAIN_STAGING)?;

    // 2. Create XRAT_KS_STG.
    run_iptables(binary, &["-N", CHAIN_STAGING])?;

    // 3. Populate XRAT_KS_STG with all ACCEPT rules.
    run_iptables(binary, &["-A", CHAIN_STAGING, "-o", "lo", "-j", "ACCEPT"])?;

    let tun = config.tun_interface.trim();
    if !tun.is_empty() {
        run_iptables(binary, &["-A", CHAIN_STAGING, "-o", tun, "-j", "ACCEPT"])?;
    }

    if let Some(mark) = config.socket_mark.filter(|m| *m > 0) {
        let mark_hex = format!("{mark:#x}");
        run_iptables(
            binary,
            &[
                "-A",
                CHAIN_STAGING,
                "-m",
                "mark",
                "--mark",
                &mark_hex,
                "-j",
                "ACCEPT",
            ],
        )?;
    }

    if is_v4 {
        for endpoint in &config.allowed_endpoints {
            if let std::net::SocketAddr::V4(v4) = endpoint {
                let ip_str = v4.ip().to_string();
                let port_str = v4.port().to_string();
                for proto in ["tcp", "udp"] {
                    run_iptables(
                        binary,
                        &[
                            "-A",
                            CHAIN_STAGING,
                            "-d",
                            &ip_str,
                            "-p",
                            proto,
                            "--dport",
                            &port_str,
                            "-j",
                            "ACCEPT",
                        ],
                    )?;
                }
            }
        }

        run_iptables(
            binary,
            &[
                "-A",
                CHAIN_STAGING,
                "-p",
                "udp",
                "--sport",
                "68",
                "--dport",
                "67",
                "-j",
                "ACCEPT",
            ],
        )?;

        if config.allow_lan {
            for cidr in [
                "10.0.0.0/8",
                "172.16.0.0/12",
                "192.168.0.0/16",
                "169.254.0.0/16",
                "224.0.0.0/4",
                "255.255.255.255/32",
            ] {
                run_iptables(binary, &["-A", CHAIN_STAGING, "-d", cidr, "-j", "ACCEPT"])?;
            }
        }
    } else {
        if !config.block_ipv6 {
            for endpoint in &config.allowed_endpoints {
                if let std::net::SocketAddr::V6(v6) = endpoint {
                    let ip_str = v6.ip().to_string();
                    let port_str = v6.port().to_string();
                    for proto in ["tcp", "udp"] {
                        run_iptables(
                            binary,
                            &[
                                "-A",
                                CHAIN_STAGING,
                                "-d",
                                &ip_str,
                                "-p",
                                proto,
                                "--dport",
                                &port_str,
                                "-j",
                                "ACCEPT",
                            ],
                        )?;
                    }
                }
            }
        }

        run_iptables(
            binary,
            &[
                "-A",
                CHAIN_STAGING,
                "-p",
                "udp",
                "--sport",
                "546",
                "--dport",
                "547",
                "-j",
                "ACCEPT",
            ],
        )?;

        for icmp_type in [
            "router-solicitation",
            "router-advertisement",
            "neighbor-solicitation",
            "neighbor-advertisement",
        ] {
            run_iptables(
                binary,
                &[
                    "-A",
                    CHAIN_STAGING,
                    "-p",
                    "icmpv6",
                    "--icmpv6-type",
                    icmp_type,
                    "-j",
                    "ACCEPT",
                ],
            )?;
        }

        if config.allow_lan {
            for cidr in ["fe80::/10", "fc00::/7", "ff00::/8"] {
                run_iptables(binary, &["-A", CHAIN_STAGING, "-d", cidr, "-j", "ACCEPT"])?;
            }
        }
    }

    // 4. Append final DROP rule to XRAT_KS_STG.
    run_iptables(binary, &["-A", CHAIN_STAGING, "-j", "DROP"])?;

    // 5 & 6. Staging chain swap
    let attached = check_chain_in_output(binary, CHAIN_NAME)?;
    if attached {
        // If XRAT_KS is already attached to OUTPUT:
        // - Insert XRAT_KS_STG at OUTPUT 1.
        run_iptables(binary, &["-I", "OUTPUT", "1", "-j", CHAIN_STAGING])?;
        // - Delete old XRAT_KS from OUTPUT.
        cleanup_chain(binary, CHAIN_NAME)?;
        // - Rename XRAT_KS_STG to XRAT_KS.
        run_iptables(binary, &["-E", CHAIN_STAGING, CHAIN_NAME])?;
    } else {
        // If XRAT_KS is not attached to OUTPUT:
        // - Flush/delete any unattached XRAT_KS.
        cleanup_chain(binary, CHAIN_NAME)?;
        // - Rename XRAT_KS_STG to XRAT_KS.
        run_iptables(binary, &["-E", CHAIN_STAGING, CHAIN_NAME])?;
        // - Insert XRAT_KS at OUTPUT 1.
        run_iptables(binary, &["-I", "OUTPUT", "1", "-j", CHAIN_NAME])?;
    }

    // 7. Verify attachment with check_chain_in_output.
    if !check_chain_in_output(binary, CHAIN_NAME)? {
        return Err(io::Error::other(format!(
            "failed to verify {binary} chain {CHAIN_NAME} attachment to OUTPUT"
        )));
    }

    Ok(())
}

pub fn remove() -> io::Result<()> {
    remove_v4()?;
    remove_v6()?;
    Ok(())
}

fn remove_v4() -> io::Result<()> {
    cleanup_chain("iptables", CHAIN_STAGING)?;
    cleanup_chain("iptables", CHAIN_NAME)?;
    Ok(())
}

fn remove_v6() -> io::Result<()> {
    if find_in_path("ip6tables").is_none() {
        return Ok(());
    }
    cleanup_chain("ip6tables", CHAIN_STAGING)?;
    cleanup_chain("ip6tables", CHAIN_NAME)?;
    Ok(())
}
