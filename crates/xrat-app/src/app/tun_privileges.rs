//! File capabilities required for managed TUN capture. The engine binary needs
//! `CAP_NET_ADMIN` to create the interface and system routes; the xrat binary
//! needs it to remove a leftover interface before launch.

use std::path::{Path, PathBuf};

use crate::app::AppError;
use crate::app::context::AppContext;

/// Capabilities granted by `xrat tun setup`.
pub const TUN_CAPABILITIES: &str = "cap_net_admin,cap_net_raw+ep";

pub struct TunPrivilegeFile {
    pub label: &'static str,
    pub path: PathBuf,
}

/// Whether this platform uses file capabilities at all. TUN privileges are a
/// Linux capability feature; other platforms need manual OS-specific setup.
pub fn capabilities_supported() -> bool {
    cfg!(target_os = "linux")
}

/// Files that need `CAP_NET_ADMIN`: the selected engine binary and the running
/// xrat binary (for stale-interface cleanup).
pub fn required_files(context: &AppContext) -> Vec<TunPrivilegeFile> {
    let engine_binary = match context.app_config.runtime.engine.as_str() {
        "v2ray" => context.runtime_paths.v2ray_path.clone(),
        "sing-box" => context.runtime_paths.sing_box_path.clone(),
        _ => context.runtime_paths.xray_path.clone(),
    };
    let engine_path = resolve_executable(&engine_binary).unwrap_or(engine_binary);
    let mut files = vec![TunPrivilegeFile {
        label: "engine file",
        path: engine_path,
    }];
    if let Ok(current) = std::env::current_exe() {
        files.push(TunPrivilegeFile {
            label: "xrat file",
            path: current,
        });
    }
    files
}

/// Resolve a configured binary to an absolute path. Absolute paths are kept as
/// is; bare names are looked up on `PATH`, because `setcap` needs a real file.
pub fn resolve_executable(path: &Path) -> Option<PathBuf> {
    if path.is_absolute() {
        return path.is_file().then(|| path.to_path_buf());
    }
    let search_path = std::env::var_os("PATH")?;
    std::env::split_paths(&search_path)
        .map(|directory| directory.join(path))
        .find(|candidate| candidate.is_file())
}

/// Effective file capabilities as reported by `getcap`. Returns `None` when the
/// platform has no file capabilities, `getcap` is unavailable, or reading failed,
/// so callers can treat an unknown result as non-blocking.
pub fn file_capabilities(path: &Path) -> Option<String> {
    file_capabilities_with_spawner(
        path,
        std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
    )
}

fn file_capabilities_with_spawner(
    path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) -> Option<String> {
    if !capabilities_supported() {
        return None;
    }
    let output = xrat_support::process::Command::with_spawner("getcap", spawner)
        .arg(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn has_net_admin(capabilities: &str) -> bool {
    capabilities.contains("cap_net_admin")
}

/// Fail before launch when the engine binary is known to lack `CAP_NET_ADMIN`.
/// An unknown result is allowed; the native validation or spawn reports the real
/// failure in that case.
pub fn ensure_engine_capability_with_spawner(
    binary_path: &Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) -> crate::app::Result<()> {
    if !capabilities_supported() {
        return Ok(());
    }
    let Some(capabilities) = file_capabilities_with_spawner(binary_path, spawner) else {
        return Ok(());
    };
    if !has_net_admin(&capabilities) {
        return Err(AppError::InvalidArgument(format!(
            "the TUN engine at {} lacks CAP_NET_ADMIN; run `xrat tun setup` to grant it",
            binary_path.display()
        )));
    }
    Ok(())
}

/// Inspect whether systemd user service blocks TUN capabilities via NoNewPrivileges=true.
#[cfg(target_os = "linux")]
pub fn systemd_service_status() -> Option<(&'static str, bool)> {
    if !capabilities_supported() {
        return None;
    }
    let user_dir =
        crate::app::commands::systemd_user_dir_with_env(&xrat_support::env::SystemEnvVars).ok()?;
    let service_file = user_dir.join("xrat-daemon.service");
    if !service_file.is_file() {
        return Some(("not installed", true));
    }
    let content = std::fs::read_to_string(&service_file).ok()?;
    if !content.contains("NoNewPrivileges=true") {
        return Some(("ready (NoNewPrivileges disabled in service)", true));
    }
    let override_file = user_dir.join("xrat-daemon.service.d").join("10-tun.conf");
    if override_file.is_file() {
        let override_content = std::fs::read_to_string(&override_file).unwrap_or_default();
        if override_content.contains("NoNewPrivileges=false") {
            return Some(("ready (NoNewPrivileges=false override present)", true));
        }
    }
    Some((
        "blocked by NoNewPrivileges=true (run `xrat tun setup` to install override)",
        false,
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn systemd_service_status() -> Option<(&'static str, bool)> {
    None
}

/// Inspect the effective capabilities and NoNewPrivs of a process by PID.
pub fn inspect_process_privileges(pid: u32) -> Option<(&'static str, bool)> {
    #[cfg(target_os = "linux")]
    {
        let status_path = format!("/proc/{pid}/status");
        let content = std::fs::read_to_string(status_path).ok()?;
        let mut no_new_privs = false;
        let mut cap_eff_has_net_admin = false;
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("NoNewPrivs:") {
                no_new_privs = val.trim() == "1";
            } else if let Some(val) = line.strip_prefix("CapEff:")
                && let Ok(mask) = u64::from_str_radix(val.trim(), 16)
            {
                cap_eff_has_net_admin = (mask & (1 << 12)) != 0;
            }
        }
        if no_new_privs {
            return Some((
                "blocked (NoNewPrivs=1 in daemon process; restart daemon via systemd)",
                false,
            ));
        }
        if !cap_eff_has_net_admin {
            return Some(("missing CAP_NET_ADMIN in running process", false));
        }
        Some((
            "ready (effective CAP_NET_ADMIN present, NoNewPrivs=0)",
            true,
        ))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{TUN_CAPABILITIES, has_net_admin, resolve_executable};

    #[test]
    fn detects_net_admin_in_capability_strings() {
        assert!(has_net_admin("cap_net_admin,cap_net_raw=ep"));
        assert!(!has_net_admin(""));
        assert!(!has_net_admin("cap_net_raw=ep"));
        assert!(TUN_CAPABILITIES.contains("cap_net_admin"));
    }

    #[test]
    fn resolves_absolute_executables_and_reports_missing() {
        let current = std::env::current_exe().expect("test executable path");
        assert_eq!(resolve_executable(&current), Some(current));
        assert_eq!(
            resolve_executable(Path::new("xrat-definitely-missing-binary")),
            None
        );
    }

    #[test]
    fn inspect_process_privileges_reports_status() {
        #[cfg(target_os = "linux")]
        {
            let res = super::inspect_process_privileges(std::process::id());
            assert!(res.is_some());
        }
    }
}
