use std::process::Command;

use crate::app::AppError;
use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::cli::{ProxyDesktopAction, ProxyDesktopKind};

use super::{ActiveEndpoints, resolve_active_endpoints};

pub(super) async fn run(
    context: &AppContext,
    action: &ProxyDesktopAction,
) -> crate::app::Result<()> {
    if !cfg!(target_os = "linux") {
        return Err(AppError::UnsupportedPlatform(
            "desktop proxy integration is Linux-only; use `xrat proxy shell enable` for terminal-only proxying".to_string(),
        ));
    }

    let desktop = match action {
        ProxyDesktopAction::Enable(args) => resolve_desktop(args.desktop)?,
        ProxyDesktopAction::Disable(args) => resolve_desktop(args.desktop)?,
        ProxyDesktopAction::Status(args) => resolve_desktop(args.desktop)?,
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
                Some(pac_url(context))
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
    }
}

fn pac_url(context: &AppContext) -> String {
    let server = &context.app_config.server;
    let host = if server.host == "0.0.0.0" || server.host.is_empty() {
        "127.0.0.1"
    } else {
        server.host.as_str()
    };
    format!("http://{host}:{}/proxy.pac", server.port)
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn manual_enable_requires_an_active_inbound() {
        assert!(gnome_enable_commands(&ActiveEndpoints::default(), None).is_err());
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
