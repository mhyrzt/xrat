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
        return Ok(
            "Would record gateway installation intent and install the closed forwarding guard.",
        );
    }
    std::fs::create_dir_all(directory)?;
    let _lock = lock(directory)?;
    verify_forwarding_disabled(interface)?;
    firewall::verify_no_flow_offload()?;
    establish_closed_guard(directory, &intent)?;
    Ok("Gateway installed with the closed IPv4 forwarding guard.")
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

pub(crate) fn recover_boot(directory: &Path) -> io::Result<()> {
    if journal::load(directory)?.is_none() {
        return Ok(());
    }
    let _lock = lock(directory)?;
    let Some(intent) = journal::load(directory)? else {
        return Ok(());
    };
    let path = directory.join(CAPTURE_FILE);
    let captured: Option<CaptureIntent> = match std::fs::read(&path) {
        Ok(bytes) => Some(serde_json::from_slice(&bytes)?),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(error) => return Err(error),
    };
    if let Some(captured) = &captured {
        if captured.installation_id != intent.installation_id {
            return Err(io::Error::other(
                "gateway capture belongs to an unknown installation",
            ));
        }
        if captured.boot_id == boot_id()? {
            return Ok(());
        }
    }
    if firewall::inspect_closed(&intent)? && captured.is_none() {
        return Ok(());
    }
    verify_forwarding_disabled(&intent.lan_interface)?;
    establish_closed_guard(directory, &intent)?;
    if captured.is_some() {
        std::fs::remove_file(path)?;
        std::fs::File::open(directory)?.sync_all()?;
    }
    Ok(())
}

pub(crate) fn activate(
    directory: &Path,
    installation_id: uuid::Uuid,
    session_id: i64,
    tun_name: &str,
) -> io::Result<()> {
    let _lock = lock(directory)?;
    let intent = journal::load(directory)?
        .filter(|intent| intent.installation_id == installation_id)
        .ok_or_else(|| io::Error::other("gateway installation changed after launch validation"))?;
    if !firewall::inspect_closed(&intent)? {
        return Err(io::Error::other(
            "gateway guard must be verified closed before activation",
        ));
    }
    firewall::verify_no_flow_offload()?;
    for device in ["all", intent.lan_interface.as_str()] {
        let value =
            std::fs::read_to_string(format!("/proc/sys/net/ipv4/conf/{device}/send_redirects"))?;
        if value.trim() != "0" {
            return Err(io::Error::other(
                "gateway activation requires IPv4 redirects disabled on all and the LAN interface",
            ));
        }
    }
    let tun = SystemTunInterfaceOps
        .inspect_interface(tun_name)?
        .filter(|interface| interface.is_tun)
        .ok_or_else(|| io::Error::other("managed TUN missing before gateway activation"))?;
    let (network, prefix) = intent
        .lan_subnet
        .split_once('/')
        .ok_or_else(|| io::Error::other("invalid gateway LAN prefix"))?;
    let invalid_prefix = || io::Error::other("invalid gateway LAN prefix");
    let capture = CaptureIntent {
        boot_id: boot_id()?,
        installation_id: intent.installation_id,
        session_id,
        tun_name: tun_name.into(),
        tun_index: NonZeroU32::new(tun.ifindex)
            .ok_or_else(|| io::Error::other("invalid TUN index"))?,
        lan_forwarding: InterfaceForwarding::inspect(&intent.lan_interface)?,
        capture_forwarding: InterfaceForwarding::inspect(tun_name)?,
    };
    let transaction = gateway::capture_transaction(
        NonZeroU32::new(intent.lan_ifindex).ok_or_else(|| io::Error::other("invalid LAN index"))?,
        capture.tun_index,
        network.parse().map_err(|_| invalid_prefix())?,
        prefix.parse().map_err(|_| invalid_prefix())?,
    )?;
    journal::create_record(directory, CAPTURE_FILE, &capture)?;
    gateway_routing::install_with_local(
        &intent.lan_interface,
        &intent.lan_subnet,
        tun_name,
        &intent.local_subnets,
    )?;
    capture.capture_forwarding.enable()?;
    capture.lan_forwarding.enable()?;
    nftables::apply_transaction(&transaction)?;
    if !firewall::inspect_active(&intent, tun_name, capture.tun_index)? {
        return Err(io::Error::other("could not verify active gateway guard"));
    }
    Ok(())
}

pub(super) fn inspect_active(
    directory: &Path,
    intent: &journal::InstallIntent,
) -> io::Result<bool> {
    let capture: CaptureIntent = match std::fs::read(directory.join(CAPTURE_FILE)) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if capture.installation_id != intent.installation_id || capture.boot_id != boot_id()? {
        return Ok(false);
    }
    firewall::inspect_active(intent, &capture.tun_name, capture.tun_index)
}

pub(crate) fn deactivate(directory: &Path, session_id: i64) -> io::Result<()> {
    let path = directory.join(CAPTURE_FILE);
    if !std::fs::exists(&path)? {
        return Ok(());
    }
    let _lock = lock(directory)?;
    let capture: CaptureIntent = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    let intent = journal::load(directory)?
        .filter(|intent| intent.installation_id == capture.installation_id)
        .ok_or_else(|| io::Error::other("gateway capture belongs to an unknown installation"))?;
    if capture.session_id != session_id {
        return Err(io::Error::other(
            "gateway capture belongs to a different runtime session",
        ));
    }
    firewall::close_capture(&intent, &capture.tun_name, capture.tun_index)?;
    capture.lan_forwarding.restore()?;
    if SystemTunInterfaceOps
        .inspect_interface(&capture.tun_name)?
        .is_some()
    {
        capture.capture_forwarding.restore()?;
    }
    gateway_routing::remove_with_local(
        &intent.lan_interface,
        &intent.lan_subnet,
        &capture.tun_name,
        &intent.local_subnets,
    )?;
    std::fs::remove_file(path)?;
    std::fs::File::open(directory)?.sync_all()
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
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
        };
        let apply = xrat_support::net::firewall::nftables::apply_transaction;
        apply("create table ip xrat_gateway { comment \"foreign\"; }\n").unwrap();
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
        assert!(!directory.path().join(CAPTURE_FILE).exists());
        assert!(firewall::inspect_closed(&intent).unwrap());
        remove(directory.path(), true).unwrap();
        assert!(firewall::inspect_closed(&intent).unwrap());
        remove(directory.path(), false).unwrap();
        assert!(journal::load(directory.path()).unwrap().is_none());
        assert!(!firewall::inspect_closed(&intent).unwrap());
        remove(directory.path(), false).unwrap();
    }
}
