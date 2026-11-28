#[cfg(target_os = "linux")]
mod active;
#[cfg(target_os = "linux")]
pub(super) use active::{close_capture, inspect_active};

mod readback;
use readback::matches_closed;
#[cfg(test)]
mod tests;

use std::io;
#[cfg(target_os = "linux")]
use std::{fmt::Write, num::NonZeroU32};

use serde_json::{Value, json};
use xrat_support::net::{SystemTunInterfaceOps, TunInterfaceOps};

use super::journal::InstallIntent;

#[cfg(target_os = "linux")]
use xrat_support::net::firewall::gateway::TABLE;
#[cfg(not(target_os = "linux"))]
const TABLE: &str = "xrat_gateway";

/// The closed baseline allows host traffic and blocks IPv4 forwarding from the
/// LAN. Activation adds narrowly scoped exceptions to this guard atomically.
#[cfg(target_os = "linux")]
fn closed_rules(intent: &InstallIntent, lan_ifindex: NonZeroU32) -> String {
    let mut rules = String::new();
    writeln!(
        rules,
        "create table ip {TABLE} {{ comment \"xrat gateway {}\"; }}",
        intent.installation_id
    )
    .unwrap();
    writeln!(
        rules,
        "add chain ip {TABLE} forward {{ type filter hook forward priority -10; policy accept; }}"
    )
    .unwrap();
    for subnet in &intent.local_subnets {
        writeln!(
            rules,
            "add rule ip {TABLE} forward meta iif {lan_ifindex} meta oif {lan_ifindex} ip saddr {} ip daddr {subnet} counter accept",
            intent.lan_subnet
        ).unwrap();
        writeln!(
            rules,
            "add rule ip {TABLE} forward meta iif {lan_ifindex} meta oif {lan_ifindex} ip saddr {subnet} ip daddr {} ct state established,related counter accept",
            intent.lan_subnet
        ).unwrap();
    }
    writeln!(
        rules,
        "add rule ip {TABLE} forward meta iif {lan_ifindex} counter drop"
    )
    .unwrap();
    rules
}

#[cfg(target_os = "linux")]
pub(super) fn install_closed(intent: &InstallIntent) -> std::io::Result<()> {
    let index = NonZeroU32::new(intent.lan_ifindex).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid LAN index")
    })?;
    for subnet in std::iter::once(&intent.lan_subnet).chain(&intent.local_subnets) {
        super::validate_subnet(subnet).map_err(io::Error::other)?;
    }
    xrat_support::net::firewall::nftables::apply_transaction(&closed_rules(intent, index))
}

pub(super) fn inspect_closed(intent: &InstallIntent) -> io::Result<bool> {
    if !matches_lan(intent)? {
        return Ok(false);
    }

    let output = xrat_support::process::Command::new("nft")
        .args(["-j", "-n", "list", "tables", "ip"])
        .output()?;
    let tables = read_objects(output)?;
    if !tables.iter().any(|object| object["table"]["name"] == TABLE) {
        return Ok(false);
    }

    let output = xrat_support::process::Command::new("nft")
        .args(["-j", "-n", "list", "table", "ip", TABLE])
        .output()?;
    Ok(matches_closed(&read_objects(output)?, intent))
}

#[cfg(target_os = "linux")]
pub(super) fn verify_no_flow_offload() -> io::Result<()> {
    let output = xrat_support::process::Command::new("nft")
        .args(["-j", "list", "ruleset"])
        .output()?;
    if read_objects(output)?
        .iter()
        .any(|object| object.get("flowtable").is_some())
    {
        return Err(io::Error::other(
            "nftables flowtable offload can bypass the gateway forwarding chain",
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub(super) fn inspect_removable_guard(intent: &InstallIntent) -> io::Result<Option<u64>> {
    if !matches_lan(intent)? {
        return Err(io::Error::other("gateway LAN interface identity changed"));
    }
    if !guard_exists()? {
        return Ok(None);
    }
    let objects = read_objects(
        xrat_support::process::Command::new("nft")
            .args(["-j", "-n", "-a", "list", "table", "ip", TABLE])
            .output()?,
    )?;
    if !matches_closed(&objects, intent) {
        return Err(io::Error::other(
            "gateway guard differs from the expected owned shape",
        ));
    }
    let handle = objects[0]["table"]["handle"]
        .as_u64()
        .ok_or_else(|| io::Error::other("gateway table handle is missing"))?;
    Ok(Some(handle))
}

#[cfg(target_os = "linux")]
pub(super) fn guard_exists() -> io::Result<bool> {
    let tables = read_objects(
        xrat_support::process::Command::new("nft")
            .args(["-j", "-n", "list", "tables", "ip"])
            .output()?,
    )?;
    Ok(tables.iter().any(|object| object["table"]["name"] == TABLE))
}

#[cfg(target_os = "linux")]
pub(super) fn remove_closed(intent: &InstallIntent) -> io::Result<()> {
    let Some(handle) = inspect_removable_guard(intent)? else {
        return Ok(());
    };
    xrat_support::net::firewall::nftables::apply_transaction(&format!(
        "delete table ip handle {handle}\n"
    ))
}

pub(super) fn matches_lan(intent: &InstallIntent) -> io::Result<bool> {
    let matching_index = SystemTunInterfaceOps
        .inspect_interface(&intent.lan_interface)?
        .is_some_and(|interface| interface.ifindex == intent.lan_ifindex && !interface.is_tun);
    if !matching_index {
        return Ok(false);
    }
    let addresses = if_addrs::get_if_addrs()?;
    let addresses = addresses
        .iter()
        .filter(|interface| interface.name == intent.lan_interface)
        .filter_map(|interface| match &interface.addr {
            if_addrs::IfAddr::V4(address) => Some(address),
            _ => None,
        })
        .collect::<Vec<_>>();
    Ok(addresses.iter().any(|address| {
        let network = std::net::Ipv4Addr::from(u32::from(address.ip) & u32::from(address.netmask));
        address.ip == intent.lan_address
            && intent.lan_subnet == format!("{network}/{}", address.prefixlen)
    }) && intent.local_subnets.iter().all(|subnet| {
        addresses.iter().any(|address| {
            let network =
                std::net::Ipv4Addr::from(u32::from(address.ip) & u32::from(address.netmask));
            *subnet == format!("{network}/{}", address.prefixlen)
        })
    }))
}

fn read_objects(output: std::process::Output) -> io::Result<Vec<Value>> {
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "nftables inspection failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    #[derive(serde::Deserialize)]
    struct Ruleset {
        nftables: Vec<Value>,
    }

    let ruleset: Ruleset = serde_json::from_slice(&output.stdout)?;
    Ok(ruleset
        .nftables
        .into_iter()
        .filter(|object| object.get("metainfo").is_none())
        .collect())
}
