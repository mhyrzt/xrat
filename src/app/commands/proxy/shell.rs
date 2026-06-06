use crate::app::AppError;
use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::cli::{ProxyShellAction, ProxyShellKind};

use super::{ActiveEndpoints, resolve_active_endpoints};

pub(super) async fn run(context: &AppContext, action: &ProxyShellAction) -> crate::app::Result<()> {
    match action {
        ProxyShellAction::Enable(args) => {
            let active = resolve_active_endpoints(context).await?;
            let (http_proxy, all_proxy) = proxy_urls(&active)?;
            let kind = detect_shell(args.shell);
            print!("{}", enable_script(kind, &http_proxy, &all_proxy));
            Ok(())
        }
        ProxyShellAction::Disable(args) => {
            let kind = detect_shell(args.shell);
            print!("{}", disable_script(kind));
            Ok(())
        }
        ProxyShellAction::Status(_) => {
            let active = resolve_active_endpoints(context).await?;
            print_status(&active);
            Ok(())
        }
    }
}

/// Resolve the `http_proxy`/`https_proxy` and `all_proxy` URLs from active
/// endpoints. HTTP is preferred for `http_proxy`/`https_proxy` (SOCKS fallback);
/// SOCKS is preferred for `all_proxy` (HTTP fallback). Errors if neither inbound
/// is active.
fn proxy_urls(active: &ActiveEndpoints) -> crate::app::Result<(String, String)> {
    let http_url = active
        .http
        .as_ref()
        .map(|(host, port)| http_proxy_url(host, *port))
        .or_else(|| {
            active
                .socks
                .as_ref()
                .map(|(host, port)| socks_proxy_url(host, *port))
        });
    let all_url = active
        .socks
        .as_ref()
        .map(|(host, port)| socks_proxy_url(host, *port))
        .or_else(|| {
            active
                .http
                .as_ref()
                .map(|(host, port)| http_proxy_url(host, *port))
        });

    match (http_url, all_url) {
        (Some(http_url), Some(all_url)) => Ok((http_url, all_url)),
        _ => Err(AppError::InvalidArgument(
            "no active HTTP or SOCKS inbound; start a runtime with `xrat connect <id>`".to_string(),
        )),
    }
}

fn loopback(host: &str) -> &str {
    if host == "0.0.0.0" || host.is_empty() {
        "127.0.0.1"
    } else {
        host
    }
}

fn http_proxy_url(host: &str, port: u16) -> String {
    format!("http://{}:{port}", loopback(host))
}

fn socks_proxy_url(host: &str, port: u16) -> String {
    format!("socks5://{}:{port}", loopback(host))
}

const VARS: [&str; 3] = ["http_proxy", "https_proxy", "all_proxy"];

fn enable_script(kind: ProxyShellKind, http_proxy: &str, all_proxy: &str) -> String {
    let value_for = |name: &str| -> &str {
        if name == "all_proxy" {
            all_proxy
        } else {
            http_proxy
        }
    };

    let mut out = String::new();
    for name in VARS {
        let value = value_for(name);
        let upper = name.to_uppercase();
        match kind {
            ProxyShellKind::Bash | ProxyShellKind::Zsh => {
                out.push_str(&format!("export {name}=\"{value}\"\n"));
                out.push_str(&format!("export {upper}=\"{value}\"\n"));
            }
            ProxyShellKind::Fish => {
                out.push_str(&format!("set -gx {name} \"{value}\"\n"));
                out.push_str(&format!("set -gx {upper} \"{value}\"\n"));
            }
        }
    }
    out
}

fn disable_script(kind: ProxyShellKind) -> String {
    let mut out = String::new();
    for name in VARS {
        let upper = name.to_uppercase();
        match kind {
            ProxyShellKind::Bash | ProxyShellKind::Zsh => {
                out.push_str(&format!("unset {name}\n"));
                out.push_str(&format!("unset {upper}\n"));
            }
            ProxyShellKind::Fish => {
                out.push_str(&format!("set -e {name}\n"));
                out.push_str(&format!("set -e {upper}\n"));
            }
        }
    }
    out
}

/// Detect the target shell: explicit override, then `$SHELL`, then the parent
/// process name via `/proc/<ppid>/comm`, defaulting to bash.
fn detect_shell(override_kind: Option<ProxyShellKind>) -> ProxyShellKind {
    if let Some(kind) = override_kind {
        return kind;
    }
    if let Some(kind) = std::env::var("SHELL")
        .ok()
        .as_deref()
        .and_then(shell_from_name)
    {
        return kind;
    }
    if let Some(kind) = parent_process_name().as_deref().and_then(shell_from_name) {
        return kind;
    }
    ProxyShellKind::Bash
}

fn shell_from_name(name: &str) -> Option<ProxyShellKind> {
    let base = name.rsplit('/').next().unwrap_or(name).trim();
    let base = base.strip_suffix(".exe").unwrap_or(base);
    match base {
        b if b.contains("fish") => Some(ProxyShellKind::Fish),
        b if b.contains("zsh") => Some(ProxyShellKind::Zsh),
        b if b.contains("bash") => Some(ProxyShellKind::Bash),
        _ => None,
    }
}

fn parent_process_name() -> Option<String> {
    let ppid = std::os::unix::process::parent_id();
    std::fs::read_to_string(format!("/proc/{ppid}/comm"))
        .ok()
        .map(|name| name.trim().to_string())
}

fn print_status(active: &ActiveEndpoints) {
    let color = output::color_enabled();
    let current = std::env::var("http_proxy")
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("all_proxy"))
        .or_else(|_| std::env::var("ALL_PROXY"))
        .ok();

    let active_hosts: Vec<String> = [active.http.as_ref(), active.socks.as_ref()]
        .into_iter()
        .flatten()
        .map(|(host, port)| format!("{}:{port}", loopback(host)))
        .collect();

    let pointing = current
        .as_deref()
        .map(|value| active_hosts.iter().any(|hostport| value.contains(hostport)))
        .unwrap_or(false);

    let state = match (&current, pointing) {
        (Some(_), true) => "shell points at active xrat endpoints",
        (Some(_), false) => "shell proxy is set but does not match active xrat endpoints",
        (None, _) => "shell has no proxy environment set",
    };

    println!(
        "{}",
        output::format_kv(
            Some("Proxy shell"),
            &[
                ("status", state.to_string()),
                (
                    "http_proxy",
                    current.clone().unwrap_or_else(|| "-".to_string())
                ),
                (
                    "active",
                    if active_hosts.is_empty() {
                        "-".to_string()
                    } else {
                        active_hosts.join(", ")
                    },
                ),
            ],
            color,
        )
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn endpoints(http: bool, socks: bool) -> ActiveEndpoints {
        ActiveEndpoints {
            http: http.then(|| ("127.0.0.1".to_string(), 18201)),
            socks: socks.then(|| ("127.0.0.1".to_string(), 18200)),
            shadowsocks: None,
        }
    }

    #[test]
    fn prefers_http_for_http_proxy_and_socks_for_all_proxy() {
        let (http_proxy, all_proxy) = proxy_urls(&endpoints(true, true)).expect("both active");
        assert_eq!(http_proxy, "http://127.0.0.1:18201");
        assert_eq!(all_proxy, "socks5://127.0.0.1:18200");
    }

    #[test]
    fn falls_back_when_only_socks_active() {
        let (http_proxy, all_proxy) = proxy_urls(&endpoints(false, true)).expect("socks active");
        assert_eq!(http_proxy, "socks5://127.0.0.1:18200");
        assert_eq!(all_proxy, "socks5://127.0.0.1:18200");
    }

    #[test]
    fn falls_back_when_only_http_active() {
        let (http_proxy, all_proxy) = proxy_urls(&endpoints(true, false)).expect("http active");
        assert_eq!(http_proxy, "http://127.0.0.1:18201");
        assert_eq!(all_proxy, "http://127.0.0.1:18201");
    }

    #[test]
    fn errors_when_no_inbound_active() {
        assert!(proxy_urls(&endpoints(false, false)).is_err());
    }

    #[test]
    fn enable_script_bash_exports_lower_and_upper() {
        let script = enable_script(
            ProxyShellKind::Bash,
            "http://127.0.0.1:18201",
            "socks5://127.0.0.1:18200",
        );
        assert!(script.contains("export http_proxy=\"http://127.0.0.1:18201\""));
        assert!(script.contains("export HTTP_PROXY=\"http://127.0.0.1:18201\""));
        assert!(script.contains("export all_proxy=\"socks5://127.0.0.1:18200\""));
        assert!(script.contains("export ALL_PROXY=\"socks5://127.0.0.1:18200\""));
    }

    #[test]
    fn enable_script_fish_uses_set_gx() {
        let script = enable_script(
            ProxyShellKind::Fish,
            "http://127.0.0.1:18201",
            "socks5://127.0.0.1:18200",
        );
        assert!(script.contains("set -gx http_proxy \"http://127.0.0.1:18201\""));
        assert!(script.contains("set -gx ALL_PROXY \"socks5://127.0.0.1:18200\""));
    }

    #[test]
    fn disable_script_unsets_for_bash_and_clears_for_fish() {
        let bash = disable_script(ProxyShellKind::Zsh);
        assert!(bash.contains("unset http_proxy"));
        assert!(bash.contains("unset ALL_PROXY"));
        let fish = disable_script(ProxyShellKind::Fish);
        assert!(fish.contains("set -e http_proxy"));
        assert!(fish.contains("set -e ALL_PROXY"));
    }

    #[test]
    fn shell_detection_reads_names() {
        assert_eq!(shell_from_name("/usr/bin/fish"), Some(ProxyShellKind::Fish));
        assert_eq!(shell_from_name("/bin/zsh"), Some(ProxyShellKind::Zsh));
        assert_eq!(shell_from_name("bash"), Some(ProxyShellKind::Bash));
        assert_eq!(shell_from_name("/usr/bin/dash"), None);
    }

    #[test]
    fn override_takes_priority() {
        assert_eq!(
            detect_shell(Some(ProxyShellKind::Fish)),
            ProxyShellKind::Fish
        );
    }
}
