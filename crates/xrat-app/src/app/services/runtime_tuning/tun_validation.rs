use crate::app::config::TunSettings;

pub(crate) fn validate(tun: &TunSettings, engine: &str) -> Result<(), String> {
    validate_for_platform(tun, engine, std::env::consts::OS)
}

fn validate_for_platform(tun: &TunSettings, engine: &str, platform: &str) -> Result<(), String> {
    if !tun.enabled {
        return Ok(());
    }
    if !matches!(platform, "linux" | "macos" | "windows") {
        return Err("[runtime.tun] managed capture requires Linux, macOS or Windows".into());
    }
    if platform == "macos" {
        if engine != "sing-box" {
            return Err(
                "[runtime.tun] macOS managed capture requires sing-box; Xray TUN is Linux-only"
                    .into(),
            );
        }
        if !tun
            .interface_name
            .strip_prefix("utun")
            .is_some_and(|suffix| {
                !suffix.is_empty()
                    && suffix.bytes().all(|byte| byte.is_ascii_digit())
                    && suffix.parse::<u32>().is_ok_and(|unit| unit < u32::MAX)
            })
        {
            return Err("[runtime.tun].interface_name on macOS must be utun followed by a number, for example utun99".into());
        }
        if tun.strict_route {
            return Err("[runtime.tun].strict_route is unsupported on macOS".into());
        }
        if !tun.route_exclude_address.is_empty() {
            return Err(
                "[runtime.tun].route_exclude_address is not yet supported by managed macOS capture"
                    .into(),
            );
        }
    }
    if !matches!(engine, "xray" | "sing-box") {
        return Err("[runtime.tun] requires Xray or sing-box; V2Ray TUN is unsupported".into());
    }
    if tun.interface_name.is_empty()
        || tun.interface_name.len() > 15
        || tun
            .interface_name
            .chars()
            .any(|value| value.is_whitespace() || matches!(value, '/' | '\0'))
    {
        return Err(
            "[runtime.tun].interface_name requires 1..15 bytes without whitespace, slash or NUL"
                .into(),
        );
    }
    if !(1280..=65535).contains(&tun.mtu) {
        return Err("[runtime.tun].mtu must be 1280..65535".into());
    }
    if tun.address.is_empty() {
        return Err("[runtime.tun].address requires at least one interface CIDR".into());
    }
    for cidr in tun.address.iter().chain(&tun.route_exclude_address) {
        let parsed = cidr.split_once('/').and_then(|(address, prefix)| {
            Some((
                address.parse::<std::net::IpAddr>().ok()?,
                prefix.parse::<u8>().ok()?,
            ))
        });
        if parsed.is_none_or(|(address, prefix)| prefix > if address.is_ipv4() { 32 } else { 128 })
        {
            return Err(
                "[runtime.tun] addresses and exclusions must be valid IPv4/IPv6 CIDRs".into(),
            );
        }
    }
    if !matches!(tun.stack.as_str(), "system" | "gvisor" | "mixed") {
        return Err("[runtime.tun].stack must be system, gvisor or mixed".into());
    }
    if engine == "xray"
        && (tun.stack != "system" || tun.strict_route || !tun.route_exclude_address.is_empty())
    {
        return Err("[runtime.tun] stack selection, strict_route and route_exclude_address are sing-box-only; Xray uses its fixed native stack".into());
    }
    if platform == "windows" {
        crate::app::services::windows_tun::policy::dns_addresses(&tun.address)?;
        if engine == "sing-box" && tun.auto_route && !tun.strict_route {
            return Err("Windows sing-box managed auto-route requires strict_route=true to constrain system DNS".into());
        }
    }
    if tun.kill_switch.is_enabled() && tun.split_mode != crate::app::config::TunSplitMode::All {
        return Err(
            "[runtime.tun].kill_switch requires split_mode = \"all\" (fail-closed kill switch is incompatible with split tunneling whitelist/blacklist)".into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_requires_singbox_and_an_explicit_utun_without_linux_options() {
        let mut tun = TunSettings {
            enabled: true,
            interface_name: "utun99".into(),
            ..Default::default()
        };
        assert!(validate_for_platform(&tun, "sing-box", "macos").is_ok());
        assert!(validate_for_platform(&tun, "xray", "macos").is_err());
        tun.strict_route = true;
        assert!(validate_for_platform(&tun, "sing-box", "windows").is_ok());
        tun.strict_route = false;
        tun.interface_name = "xrat0".into();
        assert!(validate_for_platform(&tun, "sing-box", "macos").is_err());
        tun.interface_name = "utun99".into();
        tun.route_exclude_address = vec!["192.168.0.0/16".into()];
        assert!(validate_for_platform(&tun, "sing-box", "macos").is_err());
        tun.route_exclude_address.clear();
        tun.strict_route = true;
        assert!(validate_for_platform(&tun, "sing-box", "macos").is_err());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn rejects_ignored_xray_options_and_invalid_interface_names_before_launch() {
        let mut tun = TunSettings {
            enabled: true,
            ..Default::default()
        };
        assert!(validate(&tun, "xray").is_ok());
        tun.strict_route = true;
        assert!(validate(&tun, "xray").is_err());
        assert!(validate(&tun, "sing-box").is_ok());
        tun.strict_route = false;
        tun.stack = "gvisor".into();
        assert!(validate(&tun, "xray").is_err());
        tun.interface_name = "interface-name-too-long".into();
        assert!(validate(&tun, "sing-box").is_err());
        tun.enabled = false;
        assert!(validate(&tun, "v2ray").is_ok());
    }

    #[test]
    fn rejects_kill_switch_with_split_tunneling() {
        let mut tun = TunSettings {
            enabled: true,
            kill_switch: crate::app::config::TunKillSwitchMode::Auto,
            split_mode: crate::app::config::TunSplitMode::Blacklist,
            ..Default::default()
        };
        assert!(validate_for_platform(&tun, "sing-box", "linux").is_err());
        tun.split_mode = crate::app::config::TunSplitMode::All;
        assert!(validate_for_platform(&tun, "sing-box", "linux").is_ok());
    }
}
