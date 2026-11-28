use super::*;

pub(super) fn select_lan(
    settings: &mut EffectiveSettings,
    addresses: &[InterfaceAddress],
    checks: &mut Vec<Check>,
) {
    let mut candidates: Vec<&str> = addresses
        .iter()
        .filter(|a| a.candidate)
        .map(|a| a.name.as_str())
        .collect();
    candidates.sort_unstable();
    candidates.dedup();
    if settings.interface.value.is_none() && candidates.len() == 1 {
        settings.interface.value = Some(candidates[0].to_string());
        settings.interface.source = "discovery";
    }
    let Some(name) = settings.interface.value.as_deref() else {
        checks.push(check("lan_interface", "failed", format!("Cannot choose an unambiguous LAN interface; candidates: {}. Specify --interface or [gateway].interface.", if candidates.is_empty() { "none".into() } else { candidates.join(", ") })));
        return;
    };
    let mut subnets: Vec<&str> = addresses
        .iter()
        .filter(|a| a.name == name && a.candidate)
        .map(|a| a.subnet.as_str())
        .collect();
    subnets.sort_unstable();
    subnets.dedup();
    if subnets.is_empty() {
        checks.push(check(
            "lan_interface",
            "failed",
            format!("Interface {name} has no eligible active LAN IPv4 address."),
        ));
        return;
    }
    checks.push(check(
        "lan_interface",
        "incomplete",
        format!("Selected {name}; physical LAN reachability has not been verified."),
    ));
    if settings.lan_subnet.value.is_none() && subnets.len() == 1 {
        settings.lan_subnet.value = Some(subnets[0].to_string());
        settings.lan_subnet.source = "discovery";
    }
    match settings.lan_subnet.value.as_deref() {
        Some(subnet) if subnets.contains(&subnet) => checks.push(check(
            "lan_subnet",
            "incomplete",
            format!("{subnet} matches an interface address; client selection is not verified."),
        )),
        Some(subnet) => checks.push(check(
            "lan_subnet",
            "failed",
            format!("{subnet} does not match an eligible subnet on {name}."),
        )),
        None => checks.push(check(
            "lan_subnet",
            "failed",
            format!(
                "Multiple LAN subnets on {name}: {}. Specify --lan-subnet.",
                subnets.join(", ")
            ),
        )),
    }
}

pub(super) fn parse_default_routes(input: &str) -> Result<Vec<DefaultRoute>, String> {
    let mut routes = Vec::new();
    for line in input.lines().skip(1).filter(|line| !line.trim().is_empty()) {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 8 {
            return Err("Malformed /proc/net/route; cannot establish upstream routes.".into());
        }
        let flags = u32::from_str_radix(fields[3], 16).map_err(|_| "Malformed route flags")?;
        if fields[1] != "00000000" || fields[7] != "00000000" || flags & 1 == 0 {
            continue;
        }
        let gateway = u32::from_str_radix(fields[2], 16).map_err(|_| "Malformed route gateway")?;
        let metric = fields[6].parse().map_err(|_| "Malformed route metric")?;
        routes.push(DefaultRoute {
            interface: fields[0].to_string(),
            gateway: Ipv4Addr::from(gateway.to_le_bytes()),
            metric,
        });
    }
    Ok(routes)
}
