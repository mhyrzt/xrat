use super::*;

pub(crate) fn recover_boot(directory: &Path) -> io::Result<()> {
    if journal::load(directory)?.is_none() {
        return Ok(());
    }
    let _lock = lock(directory)?;
    let Some(mut intent) = journal::load(directory)? else {
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
    let current_boot = boot_id()?;
    let previous_boot = intent
        .boot_id
        .or_else(|| captured.as_ref().map(|capture| capture.boot_id));
    if previous_boot.is_some_and(|previous| previous != current_boot) && !firewall::guard_exists()?
    {
        let link = SystemTunInterfaceOps
            .inspect_interface(&intent.lan_interface)?
            .ok_or_else(|| io::Error::other("recorded gateway LAN interface is missing"))?;
        intent.lan_ifindex = link.ifindex;
        if !firewall::matches_lan(&intent)? {
            return Err(io::Error::other(
                "recorded gateway LAN address or subnet changed",
            ));
        }
        intent.boot_id = Some(current_boot);
        journal::replace(directory, &intent)?;
    }
    establish_closed_guard(directory, &intent)?;
    if captured.is_some() {
        std::fs::remove_file(path)?;
        std::fs::File::open(directory)?.sync_all()?;
    }
    Ok(())
}
