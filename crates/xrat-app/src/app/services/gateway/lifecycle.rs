mod recovery;
pub(crate) use recovery::recover_boot;

mod capture;
pub(crate) use capture::{activate, deactivate};
pub(super) use capture::{inspect_active, preflight_launch_routes};
#[cfg(test)]
mod tests;

use std::{io, num::NonZeroU32, path::Path};

use serde::{Deserialize, Serialize};
use xrat_support::net::{
    SystemTunInterfaceOps, TunInterfaceOps,
    firewall::{gateway, nftables},
    gateway_forwarding::InterfaceForwarding,
    gateway_routing,
};

use super::{firewall, journal};

fn lock(directory: &Path) -> io::Result<std::fs::File> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(directory.join("gateway.lock"))?;
    file.lock()?;
    Ok(file)
}

pub(super) fn establish_closed_guard(
    directory: &Path,
    intent: &journal::InstallIntent,
) -> io::Result<()> {
    if !firewall::matches_lan(intent)? {
        return Err(io::Error::other("gateway LAN interface identity changed"));
    }

    match journal::load(directory)? {
        None => journal::create(directory, intent)?,
        Some(existing) if existing == *intent => {
            std::fs::File::open(directory)?.sync_all()?;
        }
        Some(_) => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "gateway installation intent differs from the requested installation",
            ));
        }
    }

    if firewall::inspect_closed(intent)? {
        return Ok(());
    }
    firewall::install_closed(intent)?;
    if !firewall::inspect_closed(intent)? {
        return Err(io::Error::other(
            "could not verify the closed gateway guard; installation intent has been retained",
        ));
    }
    Ok(())
}

pub(crate) fn install(
    directory: &Path,
    settings: &super::EffectiveSettings,
    report: &super::Report,
    dry_run: bool,
) -> io::Result<&'static str> {
    let interface = settings
        .interface
        .value
        .as_deref()
        .ok_or_else(|| io::Error::other("select a gateway LAN interface"))?;
    let subnet = settings
        .lan_subnet
        .value
        .as_deref()
        .ok_or_else(|| io::Error::other("select a gateway LAN subnet"))?;
    for local in &settings.local_subnets.value {
        let prefix = local
            .rsplit_once('/')
            .and_then(|(_, prefix)| prefix.parse::<u8>().ok());
        if prefix.is_none_or(|prefix| prefix < 8)
            || !report.interfaces.iter().any(|address| {
                address.name == interface && address.subnet == *local && address.candidate
            })
        {
            return Err(io::Error::other(format!(
                "gateway local subnet {local} must be directly attached to {interface}"
            )));
        }
    }
    let mut addresses = report.interfaces.iter().filter(|address| {
        address.name == interface && address.subnet == subnet && address.candidate
    });
    let address = addresses
        .next()
        .filter(|_| addresses.next().is_none())
        .ok_or_else(|| io::Error::other("gateway LAN address is missing or ambiguous"))?;
    let link = SystemTunInterfaceOps
        .inspect_interface(interface)?
        .filter(|link| !link.is_tun)
        .ok_or_else(|| {
            io::Error::other("gateway LAN must be a physical or bridged IPv4 interface")
        })?;
    let existing = journal::load(directory)?;
    if existing.is_none() && report.checks.iter().any(|check| check.state == "failed") {
        return Err(io::Error::other(
            "resolve failed gateway preflight checks before installation",
        ));
    }
    verify_forwarding_disabled(interface)?;
    firewall::verify_no_flow_offload()?;
    let intent = journal::InstallIntent {
        installation_id: existing
            .as_ref()
            .map_or_else(uuid::Uuid::new_v4, |intent| intent.installation_id),
        lan_interface: interface.into(),
        lan_ifindex: link.ifindex,
        lan_address: address.address,
        lan_subnet: subnet.into(),
        local_subnets: settings.local_subnets.value.clone(),
        previous_ipv4_forwarding: false,
        boot_id: match &existing {
            Some(record) => record.boot_id,
            None => Some(boot_id()?),
        },
    };
    if existing
        .as_ref()
        .is_some_and(|existing| *existing != intent)
    {
        return Err(io::Error::other(
            "gateway installation intent differs from the requested installation",
        ));
    }
    if dry_run {
        verify_install_guard(&intent)?;
        return Ok(
            "Would record gateway installation intent and install the closed forwarding guard.",
        );
    }
    std::fs::create_dir_all(directory)?;
    let _lock = lock(directory)?;
    verify_forwarding_disabled(interface)?;
    firewall::verify_no_flow_offload()?;
    verify_install_guard(&intent)?;
    establish_closed_guard(directory, &intent)?;
    Ok("Gateway installed with the closed IPv4 forwarding guard.")
}

fn verify_install_guard(intent: &journal::InstallIntent) -> io::Result<()> {
    if firewall::guard_exists()? && !firewall::inspect_closed(intent)? {
        return Err(io::Error::other(
            "gateway guard table already exists with a different owner or shape",
        ));
    }
    Ok(())
}

pub(crate) fn remove(directory: &Path, dry_run: bool) -> io::Result<&'static str> {
    if journal::load(directory)?.is_none() {
        return Ok("No gateway installation is recorded.");
    }
    let _lock = if dry_run {
        None
    } else {
        Some(lock(directory)?)
    };
    let Some(intent) = journal::load(directory)? else {
        return Ok("No gateway installation is recorded.");
    };
    if std::fs::exists(directory.join(CAPTURE_FILE))? {
        return Err(io::Error::other(
            "disconnect or recover the managed gateway session before removing installation",
        ));
    }
    verify_forwarding_disabled(&intent.lan_interface)?;
    if dry_run {
        firewall::inspect_removable_guard(&intent)?;
        return Ok("Would remove the verified closed gateway guard and its installation intent.");
    }
    firewall::remove_closed(&intent)?;
    journal::clear(directory)?;
    Ok("Gateway installation removed.")
}

fn verify_forwarding_disabled(lan: &str) -> io::Result<()> {
    super::validate_interface(lan).map_err(io::Error::other)?;
    for device in ["all", "default", lan] {
        let value =
            std::fs::read_to_string(format!("/proc/sys/net/ipv4/conf/{device}/forwarding"))?;
        if value.trim() != "0" {
            return Err(io::Error::other(format!(
                "IPv4 forwarding must be disabled for {device} before changing the gateway installation"
            )));
        }
    }
    Ok(())
}

const CAPTURE_FILE: &str = "gateway-capture-v1.json";

#[derive(Deserialize, Serialize)]
struct CaptureIntent {
    boot_id: uuid::Uuid,
    installation_id: uuid::Uuid,
    session_id: i64,
    tun_name: String,
    tun_index: NonZeroU32,
    lan_forwarding: InterfaceForwarding,
    capture_forwarding: InterfaceForwarding,
}

fn boot_id() -> io::Result<uuid::Uuid> {
    std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?
        .trim()
        .parse()
        .map_err(io::Error::other)
}
