use std::path::PathBuf;

use crate::app::AppError;
use crate::app::context::AppContext;
use crate::cli::{DaemonInstallArgs, DaemonUninstallArgs};

const DAEMON_SERVICE_NAME: &str = "xrat-daemon.service";
const API_SERVICE_NAME: &str = "xrat-api.service";

fn systemd_user_dir() -> crate::app::Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok_or(AppError::MissingHomeDirectory)?;
    Ok(base.join("systemd").join("user"))
}

fn resolve_exe() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/usr/local/bin/xrat"))
}

const DAEMON_SERVICE_TEMPLATE: &str =
    include_str!("../../../packaging/systemd/xrat-daemon.service.template");
const API_SERVICE_TEMPLATE: &str =
    include_str!("../../../packaging/systemd/xrat-api.service.template");

fn render_service(template: &str, exe: &PathBuf, xrat_path: &str) -> String {
    template
        .replace("{{EXE}}", &exe.display().to_string())
        .replace("{{XRAT_PATH}}", xrat_path)
}

fn generate_daemon_service(exe: &PathBuf, xrat_path: &str) -> String {
    render_service(DAEMON_SERVICE_TEMPLATE, exe, xrat_path)
}

fn generate_api_service(exe: &PathBuf, xrat_path: &str) -> String {
    render_service(API_SERVICE_TEMPLATE, exe, xrat_path)
}

fn run_systemctl(args: &[&str]) -> std::io::Result<()> {
    let status = std::process::Command::new("systemctl")
        .arg("--user")
        .args(args)
        .status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "systemctl --user {} exited with {}",
            args.join(" "),
            status
        )));
    }
    Ok(())
}

fn systemctl_available() -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "status"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

#[cfg(not(target_os = "linux"))]
pub fn install(_context: &AppContext, _args: &DaemonInstallArgs) -> crate::app::Result<()> {
    Err(AppError::UnsupportedPlatform(
        "daemon install is only supported on Linux with systemd".to_string(),
    ))
}

#[cfg(target_os = "linux")]
pub fn install(context: &AppContext, args: &DaemonInstallArgs) -> crate::app::Result<()> {
    if !args.dry_run && !systemctl_available() {
        return Err(AppError::InvalidArgument(
            "systemd is not available; ensure `systemctl --user status` works".to_string(),
        ));
    }

    let exe = resolve_exe();
    let xrat_path = context.runtime_paths.root_dir.display().to_string();
    let service_dir = systemd_user_dir()?;

    let daemon_content = generate_daemon_service(&exe, &xrat_path);
    let api_content = generate_api_service(&exe, &xrat_path);
    let daemon_path = service_dir.join(DAEMON_SERVICE_NAME);
    let api_path = service_dir.join(API_SERVICE_NAME);

    if args.dry_run {
        println!("--- dry run: no files written ---\n");
        println!("Service directory: {}", service_dir.display());
        println!();
        println!("--- {} ---", DAEMON_SERVICE_NAME);
        print!("{daemon_content}");
        if args.with_api {
            println!();
            println!("--- {} ---", API_SERVICE_NAME);
            print!("{api_content}");
        }
        println!();
        println!("Actions that would run:");
        println!("  systemctl --user daemon-reload");
        println!("  systemctl --user enable {DAEMON_SERVICE_NAME}");
        if args.with_api {
            println!("  systemctl --user enable {API_SERVICE_NAME}");
        }
        if args.start {
            println!("  systemctl --user start {DAEMON_SERVICE_NAME}");
        }
        return Ok(());
    }

    std::fs::create_dir_all(&service_dir)?;

    std::fs::write(&daemon_path, &daemon_content)?;
    println!("Written: {}", daemon_path.display());

    if args.with_api {
        std::fs::write(&api_path, &api_content)?;
        println!("Written: {}", api_path.display());
    }

    run_systemctl(&["daemon-reload"])?;
    println!("Reloaded systemd user daemon.");

    run_systemctl(&["enable", DAEMON_SERVICE_NAME])?;
    println!("Enabled: {DAEMON_SERVICE_NAME}");

    if args.with_api {
        run_systemctl(&["enable", API_SERVICE_NAME])?;
        println!("Enabled: {API_SERVICE_NAME}");
    }

    if args.start {
        run_systemctl(&["start", DAEMON_SERVICE_NAME])?;
        println!("Started: {DAEMON_SERVICE_NAME}");
    }

    println!();
    println!("Daemon installed successfully.");
    if !args.start {
        println!("Start with: xrat daemon start");
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn uninstall(_context: &AppContext, _args: &DaemonUninstallArgs) -> crate::app::Result<()> {
    Err(AppError::UnsupportedPlatform(
        "daemon uninstall is only supported on Linux with systemd".to_string(),
    ))
}

#[cfg(target_os = "linux")]
pub fn uninstall(_context: &AppContext, args: &DaemonUninstallArgs) -> crate::app::Result<()> {
    let service_dir = systemd_user_dir()?;
    let daemon_path = service_dir.join(DAEMON_SERVICE_NAME);
    let api_path = service_dir.join(API_SERVICE_NAME);

    if args.dry_run {
        println!("--- dry run: no files removed ---\n");
        if daemon_path.exists() {
            println!("Would stop:    systemctl --user stop {DAEMON_SERVICE_NAME}");
            println!("Would disable: systemctl --user disable {DAEMON_SERVICE_NAME}");
            println!("Would remove:  {}", daemon_path.display());
        } else {
            println!("Not present: {}", daemon_path.display());
        }
        if api_path.exists() {
            println!("Would stop:    systemctl --user stop {API_SERVICE_NAME}");
            println!("Would disable: systemctl --user disable {API_SERVICE_NAME}");
            println!("Would remove:  {}", api_path.display());
        }
        println!("Would run: systemctl --user daemon-reload");
        return Ok(());
    }

    let mut removed = false;

    if daemon_path.exists() {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "stop", DAEMON_SERVICE_NAME])
            .status();
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "disable", DAEMON_SERVICE_NAME])
            .status();
        std::fs::remove_file(&daemon_path)?;
        println!("Removed: {}", daemon_path.display());
        removed = true;
    } else {
        println!("Not present: {}", daemon_path.display());
    }

    if api_path.exists() {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "stop", API_SERVICE_NAME])
            .status();
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "disable", API_SERVICE_NAME])
            .status();
        std::fs::remove_file(&api_path)?;
        println!("Removed: {}", api_path.display());
        removed = true;
    }

    if removed {
        run_systemctl(&["daemon-reload"])?;
        println!("Reloaded systemd user daemon.");
    }

    println!();
    println!("Daemon uninstalled. Config and data preserved.");

    Ok(())
}
