//! Linux/XDG desktop integration: a terminal-aware launcher, a `.desktop`
//! entry, and hicolor icons. Ported from the former `install.sh` shell logic so
//! `xrat setup` owns it regardless of how the binary was installed. Assets are
//! embedded in the binary so they are available without the release archive.

use super::report::{StepOutcome, StepStatus};
use super::steps::STEP_DESKTOP;

// Assets are vendored under the module (not referenced from packaging/ or docs/)
// so they ship in the published crate tarball, which excludes those directories.
#[cfg(target_os = "linux")]
const DESKTOP_TEMPLATE: &str = include_str!("assets/xrat.desktop");
#[cfg(target_os = "linux")]
const ICON_48: &[u8] = include_bytes!("assets/xrat-48x48.png");
#[cfg(target_os = "linux")]
const ICON_256: &[u8] = include_bytes!("assets/xrat-256x256.png");

/// The terminal emulator setup would use for the desktop launcher, if any.
/// `None` off Linux or when no known terminal is on PATH.
#[cfg(target_os = "linux")]
pub fn detected_terminal() -> Option<String> {
    select_terminal().map(|terminal| terminal.to_string())
}

#[cfg(not(target_os = "linux"))]
pub fn detected_terminal() -> Option<String> {
    None
}

#[cfg(not(target_os = "linux"))]
pub fn probe() -> StepOutcome {
    StepOutcome::new(STEP_DESKTOP, StepStatus::Skipped, false)
        .with_detail("Linux/XDG only".to_string())
}

#[cfg(not(target_os = "linux"))]
pub fn apply() -> StepOutcome {
    StepOutcome::new(STEP_DESKTOP, StepStatus::Skipped, false)
        .with_detail("Linux/XDG only".to_string())
}

#[cfg(target_os = "linux")]
pub fn probe() -> StepOutcome {
    use crate::support::platform;
    match platform::xdg_data_home() {
        Some(data) if data.join("applications").join("xrat.desktop").exists() => {
            StepOutcome::new(STEP_DESKTOP, StepStatus::AlreadyDone, false)
        }
        Some(_) => StepOutcome::new(STEP_DESKTOP, StepStatus::Missing, false)
            .with_detail("desktop launcher not installed".to_string()),
        None => StepOutcome::new(STEP_DESKTOP, StepStatus::Skipped, false)
            .with_detail("no data directory resolvable".to_string()),
    }
}

#[cfg(target_os = "linux")]
pub fn apply() -> StepOutcome {
    use crate::support::platform;

    let Some(data) = platform::xdg_data_home() else {
        return StepOutcome::new(STEP_DESKTOP, StepStatus::Skipped, false)
            .with_detail("no data directory resolvable".to_string());
    };

    let apps_dir = data.join("applications");
    let desktop_path = apps_dir.join("xrat.desktop");
    let existed = desktop_path.exists();

    let xrat_path = std::env::current_exe()
        .ok()
        .map(|exe| exe.display().to_string())
        .unwrap_or_else(|| "xrat".to_string());

    let (exec_line, terminal_value, wm_class) = match install_launcher(&xrat_path) {
        Some(launcher) => (launcher, "false", Some("xrat")),
        None => (format!("{xrat_path} tui"), "true", None),
    };

    if let Err(error) = std::fs::create_dir_all(&apps_dir) {
        return failed(error.to_string());
    }
    let contents = render_desktop(&exec_line, terminal_value, wm_class);
    if let Err(error) = std::fs::write(&desktop_path, contents) {
        return failed(error.to_string());
    }

    if let Err(error) = install_icons(&data) {
        return failed(error);
    }

    refresh_caches(&apps_dir, &data);

    let status = if existed {
        StepStatus::AlreadyDone
    } else {
        StepStatus::Done
    };
    StepOutcome::new(STEP_DESKTOP, status, false).with_detail(desktop_path.display().to_string())
}

#[cfg(target_os = "linux")]
fn failed(detail: String) -> StepOutcome {
    StepOutcome::new(STEP_DESKTOP, StepStatus::Failed, false).with_detail(detail)
}

#[cfg(target_os = "linux")]
fn render_desktop(exec_line: &str, terminal_value: &str, wm_class: Option<&str>) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut saw_wm_class = false;
    for line in DESKTOP_TEMPLATE.lines() {
        if line.starts_with("Exec=") {
            lines.push(format!("Exec={exec_line}"));
        } else if line.starts_with("Terminal=") {
            lines.push(format!("Terminal={terminal_value}"));
        } else if line.starts_with("StartupWMClass=") {
            if let Some(class) = wm_class {
                lines.push(format!("StartupWMClass={class}"));
                saw_wm_class = true;
            }
        } else {
            lines.push(line.to_string());
        }
    }
    if let Some(class) = wm_class
        && !saw_wm_class
    {
        lines.push(format!("StartupWMClass={class}"));
    }
    let mut output = lines.join("\n");
    output.push('\n');
    output
}

#[cfg(target_os = "linux")]
fn install_icons(data: &std::path::Path) -> Result<(), String> {
    let icon_root = data.join("icons").join("hicolor");
    for (size, bytes) in [("48x48", ICON_48), ("256x256", ICON_256)] {
        let dir = icon_root.join(size).join("apps");
        std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        std::fs::write(dir.join("xrat.png"), bytes).map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn refresh_caches(apps_dir: &std::path::Path, data: &std::path::Path) {
    use crate::support::platform;
    if platform::binary_on_path("update-desktop-database").is_some() {
        let _ = std::process::Command::new("update-desktop-database")
            .arg(apps_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    if platform::binary_on_path("gtk-update-icon-cache").is_some() {
        let _ = std::process::Command::new("gtk-update-icon-cache")
            .args(["-f", "-t"])
            .arg(data.join("icons").join("hicolor"))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}

/// Write a terminal-specific `xrat-desktop` wrapper next to the binary so the
/// TUI launches in its own window with a stable WM class. Returns the Exec line
/// to use, or None when no known terminal is available or the wrapper cannot be
/// written (caller falls back to `Terminal=true`).
#[cfg(target_os = "linux")]
fn install_launcher(xrat_path: &str) -> Option<String> {
    let terminal = select_terminal()?;
    let exe_dir = super::steps::exe_dir()?;
    let launcher_path = exe_dir.join("xrat-desktop");
    let script = launcher_script(terminal, xrat_path);
    std::fs::write(&launcher_path, script).ok()?;
    set_executable(&launcher_path)?;
    Some(launcher_path.display().to_string())
}

#[cfg(target_os = "linux")]
fn set_executable(path: &std::path::Path) -> Option<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path).ok()?.permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).ok()
}

#[cfg(target_os = "linux")]
fn select_terminal() -> Option<&'static str> {
    use crate::support::platform;
    let wayland = std::env::var("XDG_SESSION_TYPE")
        .map(|value| value.eq_ignore_ascii_case("wayland"))
        .unwrap_or(false);
    let candidates: &[&str] = if wayland {
        &[
            "kitty",
            "alacritty",
            "wezterm",
            "footclient",
            "foot",
            "konsole",
        ]
    } else {
        &[
            "kitty",
            "alacritty",
            "wezterm",
            "konsole",
            "gnome-terminal",
            "xterm",
        ]
    };
    candidates
        .iter()
        .find(|terminal| platform::binary_on_path(terminal).is_some())
        .copied()
}

#[cfg(target_os = "linux")]
fn launcher_script(terminal: &str, xrat_path: &str) -> String {
    let command = match terminal {
        "kitty" => format!("exec kitty --class=xrat --title=XRAT \"{xrat_path}\" tui \"$@\""),
        "alacritty" => {
            format!("exec alacritty --class xrat,xrat --title XRAT -e \"{xrat_path}\" tui \"$@\"")
        }
        "wezterm" => {
            format!(
                "exec wezterm start --always-new-process --class xrat \"{xrat_path}\" tui \"$@\""
            )
        }
        "footclient" => {
            format!("exec footclient --app-id=xrat --title=XRAT \"{xrat_path}\" tui \"$@\"")
        }
        "foot" => format!("exec foot --app-id=xrat --title=XRAT \"{xrat_path}\" tui \"$@\""),
        "konsole" => format!("exec konsole --desktopfile xrat -e \"{xrat_path}\" tui \"$@\""),
        "gnome-terminal" => {
            format!("exec gnome-terminal --class=xrat --title=XRAT -- \"{xrat_path}\" tui \"$@\"")
        }
        "xterm" => format!("exec xterm -class xrat -title XRAT -e \"{xrat_path}\" tui \"$@\""),
        _ => format!("exec \"{xrat_path}\" tui \"$@\""),
    };
    format!("#!/usr/bin/env sh\n{command}\n")
}
