//! Individual setup steps. Each step exposes a read-only `probe_*` (used by
//! `--check`) and a mutating `apply_*` (used by the guided flow). Mutating
//! steps are idempotent: re-running reports `AlreadyDone` instead of failing.

use std::path::PathBuf;

use super::report::{StepOutcome, StepStatus};
use crate::app::commands::{completions, daemon_install, init, manpage};
use crate::app::context::AppContext;
use crate::cli::DaemonInstallArgs;
use crate::support::platform;

pub const STEP_XRAY: &str = "xray";
pub const STEP_SINGBOX: &str = "sing-box";
pub const STEP_INIT: &str = "init";
pub const STEP_DAEMON: &str = "daemon";
pub const STEP_LINGER: &str = "linger";
pub const STEP_COMPLETIONS: &str = "completions";
pub const STEP_MANPAGES: &str = "man pages";
pub const STEP_DESKTOP: &str = "desktop";
pub const STEP_PATH: &str = "PATH";

const XRAY_HINT: &str = "install xray-core: https://github.com/XTLS/Xray-core/releases";
const SINGBOX_HINT: &str =
    "optional; install sing-box: https://github.com/SagerNet/sing-box/releases";

// ---------------------------------------------------------------------------
// Dependencies
// ---------------------------------------------------------------------------

pub fn probe_dependency(name: &'static str, required: bool, hint: &str) -> StepOutcome {
    match platform::binary_on_path(name) {
        Some(path) => {
            let detail = match binary_version(&path) {
                Some(version) => format!("{} ({version})", path.display()),
                None => path.display().to_string(),
            };
            StepOutcome::new(name, StepStatus::Done, required).with_detail(detail)
        }
        None => StepOutcome::new(name, StepStatus::Missing, required).with_detail(hint.to_string()),
    }
}

/// Run `<binary> version` and extract the first semantic version it prints,
/// mirroring how `install.sh` reported tool versions.
fn binary_version(path: &std::path::Path) -> Option<String> {
    let output = std::process::Command::new(path)
        .arg("version")
        .output()
        .ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    extract_semver(&text)
}

fn extract_semver(text: &str) -> Option<String> {
    for token in text.split(|ch: char| !(ch.is_ascii_digit() || ch == '.')) {
        let mut parts = token.split('.');
        let has_two_numeric_parts = token.contains('.')
            && parts.clone().count() >= 2
            && parts.all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()));
        if has_two_numeric_parts {
            return Some(format!("v{token}"));
        }
    }
    None
}

pub fn probe_xray() -> StepOutcome {
    probe_dependency(STEP_XRAY, true, XRAY_HINT)
}

pub fn probe_singbox() -> StepOutcome {
    probe_dependency(STEP_SINGBOX, false, SINGBOX_HINT)
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

pub fn probe_init(context: &AppContext) -> StepOutcome {
    if context.runtime_paths.config_path.exists() {
        StepOutcome::new(STEP_INIT, StepStatus::AlreadyDone, true)
            .with_detail(context.runtime_paths.root_dir.display().to_string())
    } else {
        StepOutcome::new(STEP_INIT, StepStatus::Missing, true)
            .with_detail("config directory not created".to_string())
    }
}

pub fn apply_init(context: &AppContext) -> StepOutcome {
    match init::initialize(context) {
        Ok(report) => {
            let status = if report.created_anything() {
                StepStatus::Done
            } else {
                StepStatus::AlreadyDone
            };
            StepOutcome::new(STEP_INIT, status, true)
                .with_detail(context.runtime_paths.root_dir.display().to_string())
        }
        Err(error) => {
            StepOutcome::new(STEP_INIT, StepStatus::Failed, true).with_detail(error.to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// daemon
// ---------------------------------------------------------------------------

pub fn daemon_unit_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        platform::xdg_config_home()
            .map(|dir| dir.join("systemd").join("user").join("xrat-daemon.service"))
    }
    #[cfg(target_os = "macos")]
    {
        platform::home_dir().map(|home| home.join("Library/LaunchAgents/com.xrat.daemon.plist"))
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

fn daemon_installed() -> bool {
    daemon_unit_path()
        .map(|path| path.exists())
        .unwrap_or(false)
}

pub fn probe_daemon() -> StepOutcome {
    if daemon_installed() {
        StepOutcome::new(STEP_DAEMON, StepStatus::AlreadyDone, false)
    } else {
        StepOutcome::new(STEP_DAEMON, StepStatus::Missing, false)
            .with_detail("background daemon not installed".to_string())
    }
}

pub fn apply_daemon(context: &AppContext) -> StepOutcome {
    let existed = daemon_installed();
    let args = DaemonInstallArgs {
        start: true,
        dry_run: false,
        with_api: false,
    };
    match daemon_install::install(context, &args, true) {
        Ok(()) => {
            if existed {
                StepOutcome::new(STEP_DAEMON, StepStatus::AlreadyDone, false)
                    .with_detail("reloaded and started".to_string())
            } else {
                StepOutcome::new(STEP_DAEMON, StepStatus::Done, false)
                    .with_detail("installed and started".to_string())
            }
        }
        Err(error) => {
            StepOutcome::new(STEP_DAEMON, StepStatus::Failed, false).with_detail(error.to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// linger (Linux only)
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
fn current_user() -> Option<String> {
    std::env::var("USER").ok().filter(|name| !name.is_empty())
}

#[cfg(target_os = "linux")]
fn linger_enabled() -> bool {
    let Some(user) = current_user() else {
        return false;
    };
    let Ok(output) = std::process::Command::new("loginctl")
        .args(["show-user", &user, "--property=Linger"])
        .output()
    else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .eq_ignore_ascii_case("Linger=yes")
}

#[cfg(target_os = "linux")]
pub fn probe_linger() -> StepOutcome {
    if linger_enabled() {
        StepOutcome::new(STEP_LINGER, StepStatus::AlreadyDone, false)
    } else {
        StepOutcome::new(STEP_LINGER, StepStatus::Missing, false)
            .with_detail("boot-before-login start not enabled".to_string())
    }
}

#[cfg(not(target_os = "linux"))]
pub fn probe_linger() -> StepOutcome {
    StepOutcome::new(STEP_LINGER, StepStatus::Skipped, false).with_detail("Linux only".to_string())
}

#[cfg(target_os = "linux")]
pub fn apply_linger() -> StepOutcome {
    if platform::binary_on_path("loginctl").is_none() {
        return StepOutcome::new(STEP_LINGER, StepStatus::Skipped, false)
            .with_detail("loginctl not found".to_string());
    }
    if linger_enabled() {
        return StepOutcome::new(STEP_LINGER, StepStatus::AlreadyDone, false);
    }
    let Some(user) = current_user() else {
        return StepOutcome::new(STEP_LINGER, StepStatus::Failed, false)
            .with_detail("could not determine current user".to_string());
    };
    match std::process::Command::new("loginctl")
        .args(["enable-linger", &user])
        .status()
    {
        Ok(status) if status.success() => StepOutcome::new(STEP_LINGER, StepStatus::Done, false),
        Ok(status) => StepOutcome::new(STEP_LINGER, StepStatus::Failed, false)
            .with_detail(format!("loginctl exited with {status}")),
        Err(error) => {
            StepOutcome::new(STEP_LINGER, StepStatus::Failed, false).with_detail(error.to_string())
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn apply_linger() -> StepOutcome {
    StepOutcome::new(STEP_LINGER, StepStatus::Skipped, false).with_detail("Linux only".to_string())
}

// ---------------------------------------------------------------------------
// completions
// ---------------------------------------------------------------------------

fn completion_targets() -> Vec<(clap_complete::Shell, PathBuf)> {
    use clap_complete::Shell;
    let mut targets = Vec::new();
    if let Some(data) = platform::xdg_data_home() {
        targets.push((
            Shell::Bash,
            data.join("bash-completion")
                .join("completions")
                .join("xrat"),
        ));
        targets.push((
            Shell::Zsh,
            data.join("zsh").join("site-functions").join("_xrat"),
        ));
    }
    if let Some(config) = platform::xdg_config_home() {
        targets.push((
            Shell::Fish,
            config.join("fish").join("completions").join("xrat.fish"),
        ));
    }
    targets
}

pub fn probe_completions() -> StepOutcome {
    let targets = completion_targets();
    if targets.is_empty() {
        return StepOutcome::new(STEP_COMPLETIONS, StepStatus::Skipped, false)
            .with_detail("no completion directories resolvable".to_string());
    }
    if targets.iter().all(|(_, path)| path.exists()) {
        StepOutcome::new(STEP_COMPLETIONS, StepStatus::AlreadyDone, false)
    } else {
        StepOutcome::new(STEP_COMPLETIONS, StepStatus::Missing, false)
            .with_detail("shell completions not fully installed".to_string())
    }
}

pub fn apply_completions() -> StepOutcome {
    let targets = completion_targets();
    if targets.is_empty() {
        return StepOutcome::new(STEP_COMPLETIONS, StepStatus::Skipped, false)
            .with_detail("no completion directories resolvable".to_string());
    }
    let preexisting = targets.iter().all(|(_, path)| path.exists());
    for (shell, path) in &targets {
        if let Some(parent) = path.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            return StepOutcome::new(STEP_COMPLETIONS, StepStatus::Failed, false)
                .with_detail(error.to_string());
        }
        let mut buffer = Vec::new();
        completions::generate_to(*shell, &mut buffer);
        if let Err(error) = std::fs::write(path, &buffer) {
            return StepOutcome::new(STEP_COMPLETIONS, StepStatus::Failed, false)
                .with_detail(error.to_string());
        }
    }
    let status = if preexisting {
        StepStatus::AlreadyDone
    } else {
        StepStatus::Done
    };
    StepOutcome::new(STEP_COMPLETIONS, status, false)
        .with_detail(format!("{} shells", targets.len()))
}

// ---------------------------------------------------------------------------
// man pages
// ---------------------------------------------------------------------------

fn man_dir() -> Option<PathBuf> {
    platform::xdg_data_home().map(|data| data.join("man").join("man1"))
}

pub fn probe_manpages() -> StepOutcome {
    match man_dir() {
        Some(dir) if dir.join("xrat.1").exists() => {
            StepOutcome::new(STEP_MANPAGES, StepStatus::AlreadyDone, false)
        }
        Some(_) => StepOutcome::new(STEP_MANPAGES, StepStatus::Missing, false)
            .with_detail("man pages not installed".to_string()),
        None => StepOutcome::new(STEP_MANPAGES, StepStatus::Skipped, false)
            .with_detail("no man directory resolvable".to_string()),
    }
}

pub fn apply_manpages() -> StepOutcome {
    let Some(dir) = man_dir() else {
        return StepOutcome::new(STEP_MANPAGES, StepStatus::Skipped, false)
            .with_detail("no man directory resolvable".to_string());
    };
    let existed = dir.join("xrat.1").exists();
    match manpage::generate_into(&dir) {
        Ok(count) => {
            let status = if existed {
                StepStatus::AlreadyDone
            } else {
                StepStatus::Done
            };
            StepOutcome::new(STEP_MANPAGES, status, false).with_detail(format!("{count} pages"))
        }
        Err(error) => StepOutcome::new(STEP_MANPAGES, StepStatus::Failed, false)
            .with_detail(error.to_string()),
    }
}

// ---------------------------------------------------------------------------
// PATH
// ---------------------------------------------------------------------------

pub fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
}

pub fn probe_path() -> StepOutcome {
    match exe_dir() {
        Some(dir) if platform::dir_in_path(&dir) => {
            StepOutcome::new(STEP_PATH, StepStatus::Done, false)
                .with_detail(dir.display().to_string())
        }
        Some(dir) => StepOutcome::new(STEP_PATH, StepStatus::Missing, false).with_detail(format!(
            "{} is not in PATH; add: export PATH=\"{}:$PATH\"",
            dir.display(),
            dir.display()
        )),
        None => StepOutcome::new(STEP_PATH, StepStatus::Skipped, false)
            .with_detail("could not resolve executable directory".to_string()),
    }
}
