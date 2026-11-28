use super::*;

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

pub(in crate::app::services::gateway) fn preflight_launch_routes(
    directory: &Path,
    intent: &journal::InstallIntent,
) -> io::Result<()> {
    let path = directory.join(CAPTURE_FILE);
    let captured: CaptureIntent = match std::fs::read(path) {
        Ok(data) => serde_json::from_slice(&data)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return gateway_routing::preflight_install();
        }
        Err(error) => return Err(error),
    };
    if captured.installation_id != intent.installation_id
        || captured.boot_id != boot_id()?
        || !firewall::inspect_active(intent, &captured.tun_name, captured.tun_index)?
    {
        return Err(io::Error::other(
            "recover the existing gateway capture before replacement",
        ));
    }
    gateway_routing::preflight_replacement(
        &intent.lan_interface,
        &intent.lan_subnet,
        &captured.tun_name,
        &intent.local_subnets,
    )
}

pub(in crate::app::services::gateway) fn inspect_active(
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
