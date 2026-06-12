use std::process::Command;

use crate::app::AppError;
use crate::app::commands::output;
use crate::app::config::ServerSettings;
use crate::app::context::AppContext;
use crate::cli::{ProxyDesktopAction, ProxyDesktopKind};

use super::{ActiveEndpoints, resolve_active_endpoints};

pub(super) async fn run(
    context: &AppContext,
    action: &ProxyDesktopAction,
) -> crate::app::Result<()> {
    #[cfg(target_os = "linux")]
    {
        run_linux(context, action).await
    }
    #[cfg(target_os = "macos")]
    {
        run_macos(context, action).await
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (context, action);
        Err(AppError::UnsupportedPlatform(
            "desktop proxy integration is not supported on this platform; use `xrat proxy shell enable` for terminal-only proxying".to_string(),
        ))
    }
}

#[cfg(target_os = "linux")]
async fn run_linux(context: &AppContext, action: &ProxyDesktopAction) -> crate::app::Result<()> {
    let desktop = match action {
        ProxyDesktopAction::Enable(args) => resolve_desktop(args.desktop)?,
        ProxyDesktopAction::Disable(args) => resolve_desktop(args.desktop)?,
        ProxyDesktopAction::Status(args) => resolve_desktop(args.desktop)?,
        ProxyDesktopAction::Toggle(args) => resolve_desktop(args.desktop)?,
    };

    // Only GNOME (gsettings) is supported for now.
    if desktop != ProxyDesktopKind::Gnome {
        return Err(AppError::UnsupportedPlatform(format!(
            "{desktop:?} desktop proxy integration is not supported yet; use `xrat proxy shell enable` for terminal-only proxying"
        )));
    }

    match action {
        ProxyDesktopAction::Enable(args) => {
            let pac_url = if args.pac {
                Some(pac_url(context)?)
            } else {
                None
            };
            let active = resolve_active_endpoints(context).await?;
            let commands = gnome_enable_commands(&active, pac_url.as_deref())?;
            run_gsettings_batch(&commands)?;
            println!(
                "{}",
                output::success("Desktop proxy enabled (GNOME).", output::color_enabled())
            );
            Ok(())
        }
        ProxyDesktopAction::Disable(_) => {
            run_gsettings_batch(&gnome_disable_commands())?;
            println!(
                "{}",
                output::success("Desktop proxy disabled (GNOME).", output::color_enabled())
            );
            Ok(())
        }
        ProxyDesktopAction::Status(_) => {
            let mode = run_gsettings_get(&["org.gnome.system.proxy", "mode"])?;
            println!(
                "{}",
                output::format_kv(
                    Some("Desktop proxy (GNOME)"),
                    &[("mode", mode.trim().trim_matches('\'').to_string())],
                    output::color_enabled(),
                )
            );
            Ok(())
        }
        ProxyDesktopAction::Toggle(args) => {
            let mode = run_gsettings_get(&["org.gnome.system.proxy", "mode"])?;
            if normalize_gsettings_value(&mode) == "none" {
                let pac_url = args.pac.then(|| pac_url(context)).transpose()?;
                let active = resolve_active_endpoints(context).await?;
                run_gsettings_batch(&gnome_toggle_commands(&active, pac_url.as_deref(), &mode)?)?;
                println!(
                    "{}",
                    output::success("Desktop proxy enabled (GNOME).", output::color_enabled())
                );
            } else {
                run_gsettings_batch(&gnome_toggle_commands(
                    &ActiveEndpoints::default(),
                    None,
                    &mode,
                )?)?;
                println!(
                    "{}",
                    output::success("Desktop proxy disabled (GNOME).", output::color_enabled())
                );
            }
            Ok(())
        }
    }
}

fn pac_url(context: &AppContext) -> crate::app::Result<String> {
    pac_url_from_server(&context.app_config.server)
}

fn pac_url_from_server(server: &ServerSettings) -> crate::app::Result<String> {
    if !server.enabled {
        return Err(AppError::InvalidArgument(
            "API server is disabled; enable [server] before using `xrat proxy desktop enable --pac`"
                .to_string(),
        ));
    }
    if !server.pac_enabled {
        return Err(AppError::InvalidArgument(
            "PAC serving is disabled; set [server].pac_enabled = true before using `xrat proxy desktop enable --pac`"
                .to_string(),
        ));
    }

    let host = if server.host == "0.0.0.0" || server.host.is_empty() {
        "127.0.0.1"
    } else {
        server.host.as_str()
    };
    Ok(format!("http://{host}:{}/proxy.pac", server.port))
}

fn loopback(host: &str) -> &str {
    if host == "0.0.0.0" || host.is_empty() {
        "127.0.0.1"
    } else {
        host
    }
}

/// Build the gsettings argument vectors to enable a GNOME proxy from active
/// endpoints, or a PAC autoconfig URL when `pac_url` is set.
fn gnome_enable_commands(
    active: &ActiveEndpoints,
    pac_url: Option<&str>,
) -> crate::app::Result<Vec<Vec<String>>> {
    if let Some(url) = pac_url {
        return Ok(vec![
            set_cmd("org.gnome.system.proxy", "mode", "auto"),
            set_cmd("org.gnome.system.proxy", "autoconfig-url", url),
        ]);
    }

    if active.http.is_none() && active.socks.is_none() {
        return Err(AppError::InvalidArgument(
            "no active HTTP or SOCKS inbound; start a runtime with `xrat connect <id>`".to_string(),
        ));
    }

    let mut commands = vec![set_cmd("org.gnome.system.proxy", "mode", "manual")];
    if let Some((host, port)) = &active.http {
        let host = loopback(host).to_string();
        commands.push(set_cmd("org.gnome.system.proxy.http", "host", &host));
        commands.push(set_cmd(
            "org.gnome.system.proxy.http",
            "port",
            &port.to_string(),
        ));
        commands.push(set_cmd("org.gnome.system.proxy.https", "host", &host));
        commands.push(set_cmd(
            "org.gnome.system.proxy.https",
            "port",
            &port.to_string(),
        ));
    }
    if let Some((host, port)) = &active.socks {
        let host = loopback(host).to_string();
        commands.push(set_cmd("org.gnome.system.proxy.socks", "host", &host));
        commands.push(set_cmd(
            "org.gnome.system.proxy.socks",
            "port",
            &port.to_string(),
        ));
    }
    Ok(commands)
}

fn gnome_disable_commands() -> Vec<Vec<String>> {
    vec![set_cmd("org.gnome.system.proxy", "mode", "none")]
}

fn gnome_toggle_commands(
    active: &ActiveEndpoints,
    pac_url: Option<&str>,
    current_mode: &str,
) -> crate::app::Result<Vec<Vec<String>>> {
    if normalize_gsettings_value(current_mode) == "none" {
        gnome_enable_commands(active, pac_url)
    } else {
        Ok(gnome_disable_commands())
    }
}

fn normalize_gsettings_value(value: &str) -> String {
    value.trim().trim_matches('\'').to_string()
}

fn set_cmd(schema: &str, key: &str, value: &str) -> Vec<String> {
    vec![
        "set".to_string(),
        schema.to_string(),
        key.to_string(),
        value.to_string(),
    ]
}

fn run_gsettings_batch(commands: &[Vec<String>]) -> crate::app::Result<()> {
    for args in commands {
        run_gsettings(args)?;
    }
    Ok(())
}

fn run_gsettings(args: &[String]) -> crate::app::Result<()> {
    let status = Command::new("gsettings")
        .args(args)
        .status()
        .map_err(|err| {
            AppError::InvalidArgument(format!(
                "could not run gsettings (is GNOME installed?): {err}"
            ))
        })?;
    if !status.success() {
        return Err(AppError::InvalidArgument(format!(
            "gsettings {} failed ({status})",
            args.join(" ")
        )));
    }
    Ok(())
}

fn run_gsettings_get(args: &[&str]) -> crate::app::Result<String> {
    let mut full = vec!["get"];
    full.extend_from_slice(args);
    let output = Command::new("gsettings")
        .args(&full)
        .output()
        .map_err(|err| {
            AppError::InvalidArgument(format!(
                "could not run gsettings (is GNOME installed?): {err}"
            ))
        })?;
    if !output.status.success() {
        return Err(AppError::InvalidArgument(
            "gsettings get failed".to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Auto-detect the desktop environment from common session env vars.
fn resolve_desktop(
    override_kind: Option<ProxyDesktopKind>,
) -> crate::app::Result<ProxyDesktopKind> {
    if let Some(kind) = override_kind {
        return Ok(kind);
    }

    let hint = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default()
        .to_lowercase();

    desktop_from_hint(&hint).ok_or_else(|| {
        AppError::UnsupportedPlatform(
            "could not detect the desktop environment; pass --desktop gnome|kde|xfce, or use `xrat proxy shell enable`".to_string(),
        )
    })
}

fn desktop_from_hint(hint: &str) -> Option<ProxyDesktopKind> {
    if hint.contains("gnome") || hint.contains("unity") || hint.contains("cinnamon") {
        Some(ProxyDesktopKind::Gnome)
    } else if hint.contains("kde") || hint.contains("plasma") {
        Some(ProxyDesktopKind::Kde)
    } else if hint.contains("xfce") {
        Some(ProxyDesktopKind::Xfce)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// macOS: networksetup
// ---------------------------------------------------------------------------

/// Parse `networksetup -listallnetworkservices` output into enabled service
/// names. The first line is a header, and disabled services are prefixed `*`.
#[cfg(any(target_os = "macos", test))]
fn parse_network_services(output: &str) -> Vec<String> {
    output
        .lines()
        .skip(1)
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('*'))
        .map(str::to_string)
        .collect()
}

#[cfg(target_os = "macos")]
async fn run_macos(context: &AppContext, action: &ProxyDesktopAction) -> crate::app::Result<()> {
    match action {
        ProxyDesktopAction::Enable(args) => {
            macos_apply_enable(context, args.pac).await?;
            println!(
                "{}",
                output::success("Desktop proxy enabled (macOS).", output::color_enabled())
            );
            Ok(())
        }
        ProxyDesktopAction::Disable(_) => {
            for service in &active_network_services()? {
                networksetup_disable(service)?;
            }
            println!(
                "{}",
                output::success("Desktop proxy disabled (macOS).", output::color_enabled())
            );
            Ok(())
        }
        ProxyDesktopAction::Status(_) => {
            for service in &active_network_services()? {
                let web = networksetup_output(&["-getwebproxy", service])?;
                let socks = networksetup_output(&["-getsocksfirewallproxy", service])?;
                println!(
                    "{}",
                    output::format_kv(
                        Some(&format!("Desktop proxy ({service})")),
                        &[
                            ("web proxy", proxy_enabled_label(&web)),
                            ("socks proxy", proxy_enabled_label(&socks)),
                        ],
                        output::color_enabled(),
                    )
                );
            }
            Ok(())
        }
        ProxyDesktopAction::Toggle(args) => {
            let services = active_network_services()?;
            let currently_on = services
                .iter()
                .any(|service| macos_service_proxy_on(service));
            if currently_on {
                for service in &services {
                    networksetup_disable(service)?;
                }
                println!(
                    "{}",
                    output::success("Desktop proxy disabled (macOS).", output::color_enabled())
                );
            } else {
                macos_apply_enable(context, args.pac).await?;
                println!(
                    "{}",
                    output::success("Desktop proxy enabled (macOS).", output::color_enabled())
                );
            }
            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
async fn macos_apply_enable(context: &AppContext, pac: bool) -> crate::app::Result<()> {
    let services = active_network_services()?;
    if pac {
        let url = pac_url(context)?;
        for service in &services {
            networksetup_set_pac(service, &url)?;
        }
        return Ok(());
    }

    let active = resolve_active_endpoints(context).await?;
    if active.http.is_none() && active.socks.is_none() {
        return Err(AppError::InvalidArgument(
            "no active HTTP or SOCKS inbound; start a runtime with `xrat connect <id>`".to_string(),
        ));
    }
    for service in &services {
        networksetup_enable(service, &active)?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_service_proxy_on(service: &str) -> bool {
    let web = networksetup_output(&["-getwebproxy", service]).unwrap_or_default();
    let socks = networksetup_output(&["-getsocksfirewallproxy", service]).unwrap_or_default();
    proxy_enabled_label(&web) == "Yes" || proxy_enabled_label(&socks) == "Yes"
}

#[cfg(target_os = "macos")]
fn proxy_enabled_label(output: &str) -> String {
    output
        .lines()
        .find_map(|line| line.trim().strip_prefix("Enabled: "))
        .unwrap_or("?")
        .trim()
        .to_string()
}

#[cfg(target_os = "macos")]
fn active_network_services() -> crate::app::Result<Vec<String>> {
    let out = networksetup_output(&["-listallnetworkservices"])?;
    let services = parse_network_services(&out);
    if services.is_empty() {
        return Err(AppError::InvalidArgument(
            "no active network services found via networksetup".to_string(),
        ));
    }
    Ok(services)
}

#[cfg(target_os = "macos")]
fn networksetup_enable(service: &str, active: &ActiveEndpoints) -> crate::app::Result<()> {
    if let Some((host, port)) = &active.http {
        let host = loopback(host);
        let port = port.to_string();
        run_networksetup(&["-setwebproxy", service, host, &port])?;
        run_networksetup(&["-setsecurewebproxy", service, host, &port])?;
    }
    if let Some((host, port)) = &active.socks {
        let host = loopback(host);
        let port = port.to_string();
        run_networksetup(&["-setsocksfirewallproxy", service, host, &port])?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn networksetup_disable(service: &str) -> crate::app::Result<()> {
    run_networksetup(&["-setwebproxystate", service, "off"])?;
    run_networksetup(&["-setsecurewebproxystate", service, "off"])?;
    run_networksetup(&["-setsocksfirewallproxystate", service, "off"])?;
    run_networksetup(&["-setautoproxystate", service, "off"])?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn networksetup_set_pac(service: &str, url: &str) -> crate::app::Result<()> {
    run_networksetup(&["-setautoproxyurl", service, url])?;
    run_networksetup(&["-setautoproxystate", service, "on"])?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn run_networksetup(args: &[&str]) -> crate::app::Result<()> {
    let status = Command::new("networksetup")
        .args(args)
        .status()
        .map_err(|err| AppError::InvalidArgument(format!("could not run networksetup: {err}")))?;
    if !status.success() {
        return Err(AppError::InvalidArgument(format!(
            "networksetup {} failed ({status})",
            args.join(" ")
        )));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn networksetup_output(args: &[&str]) -> crate::app::Result<String> {
    let output = Command::new("networksetup")
        .args(args)
        .output()
        .map_err(|err| AppError::InvalidArgument(format!("could not run networksetup: {err}")))?;
    if !output.status.success() {
        return Err(AppError::InvalidArgument(format!(
            "networksetup {} failed",
            args.join(" ")
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::AppConfig;

    fn endpoints() -> ActiveEndpoints {
        ActiveEndpoints {
            http: Some(("127.0.0.1".to_string(), 18201)),
            socks: Some(("127.0.0.1".to_string(), 18200)),
            shadowsocks: None,
        }
    }

    #[test]
    fn manual_enable_sets_mode_and_hosts() {
        let commands = gnome_enable_commands(&endpoints(), None).expect("commands");
        assert_eq!(
            commands[0],
            vec!["set", "org.gnome.system.proxy", "mode", "manual"]
        );
        assert!(commands.iter().any(|c| c
            == &vec![
                "set".to_string(),
                "org.gnome.system.proxy.socks".to_string(),
                "port".to_string(),
                "18200".to_string()
            ]));
    }

    #[test]
    fn pac_enable_uses_auto_mode() {
        let commands = gnome_enable_commands(
            &ActiveEndpoints::default(),
            Some("http://127.0.0.1:8787/proxy.pac"),
        )
        .expect("commands");
        assert_eq!(
            commands[0],
            vec!["set", "org.gnome.system.proxy", "mode", "auto"]
        );
        assert_eq!(
            commands[1],
            vec![
                "set",
                "org.gnome.system.proxy",
                "autoconfig-url",
                "http://127.0.0.1:8787/proxy.pac"
            ]
        );
    }

    #[test]
    fn pac_url_requires_server_and_pac_enabled() {
        let mut config = AppConfig::default();
        config.server.enabled = true;
        config.server.pac_enabled = false;
        assert!(pac_url_from_server(&config.server).is_err());

        config.server.enabled = false;
        config.server.pac_enabled = true;
        assert!(pac_url_from_server(&config.server).is_err());

        config.server.enabled = true;
        assert_eq!(
            pac_url_from_server(&config.server).expect("pac url"),
            "http://127.0.0.1:18203/proxy.pac"
        );
    }

    #[test]
    fn manual_enable_requires_an_active_inbound() {
        assert!(gnome_enable_commands(&ActiveEndpoints::default(), None).is_err());
    }

    #[test]
    fn toggle_enables_when_mode_is_none() {
        let commands = gnome_toggle_commands(&endpoints(), None, "'none'\n").expect("commands");
        assert_eq!(
            commands[0],
            vec!["set", "org.gnome.system.proxy", "mode", "manual"]
        );
    }

    #[test]
    fn toggle_disables_when_mode_is_manual_or_auto() {
        for mode in ["'manual'", "'auto'"] {
            assert_eq!(
                gnome_toggle_commands(&endpoints(), None, mode).expect("commands"),
                gnome_disable_commands()
            );
        }
    }

    #[test]
    fn toggle_can_enable_with_pac() {
        let commands = gnome_toggle_commands(
            &ActiveEndpoints::default(),
            Some("http://127.0.0.1:8787/proxy.pac"),
            "none",
        )
        .expect("commands");
        assert_eq!(
            commands[0],
            vec!["set", "org.gnome.system.proxy", "mode", "auto"]
        );
    }

    #[test]
    fn parses_enabled_network_services() {
        let output = "An asterisk (*) denotes that a network service is disabled.\nWi-Fi\nThunderbolt Bridge\n*Disabled Service\n";
        assert_eq!(
            parse_network_services(output),
            vec!["Wi-Fi".to_string(), "Thunderbolt Bridge".to_string()]
        );
    }

    #[test]
    fn detects_desktops_from_hints() {
        assert_eq!(desktop_from_hint("gnome"), Some(ProxyDesktopKind::Gnome));
        assert_eq!(
            desktop_from_hint("ubuntu:gnome"),
            Some(ProxyDesktopKind::Gnome)
        );
        assert_eq!(desktop_from_hint("kde"), Some(ProxyDesktopKind::Kde));
        assert_eq!(desktop_from_hint("xfce"), Some(ProxyDesktopKind::Xfce));
        assert_eq!(desktop_from_hint("sway"), None);
    }
}
