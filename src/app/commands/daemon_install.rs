use std::path::{Path, PathBuf};

use crate::app::AppError;
use crate::app::context::AppContext;
use crate::cli::{DaemonInstallArgs, DaemonUninstallArgs};

fn resolve_exe() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("/usr/local/bin/xrat"))
}

fn render_service(template: &str, exe: &Path, xrat_path: &str) -> String {
    template
        .replace("{{EXE}}", &exe.display().to_string())
        .replace("{{XRAT_PATH}}", xrat_path)
}

// ---------------------------------------------------------------------------
// Linux: systemd user services
// ---------------------------------------------------------------------------

#[cfg(target_os = "linux")]
const DAEMON_SERVICE_NAME: &str = "xrat-daemon.service";
#[cfg(target_os = "linux")]
const API_SERVICE_NAME: &str = "xrat-api.service";

#[cfg(target_os = "linux")]
const DAEMON_SERVICE_TEMPLATE: &str =
    include_str!("../../../packaging/systemd/xrat-daemon.service.template");
#[cfg(target_os = "linux")]
const API_SERVICE_TEMPLATE: &str =
    include_str!("../../../packaging/systemd/xrat-api.service.template");

#[cfg(target_os = "linux")]
fn systemd_user_dir() -> crate::app::Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok_or(AppError::MissingHomeDirectory)?;
    Ok(base.join("systemd").join("user"))
}

#[cfg(target_os = "linux")]
fn generate_daemon_service(exe: &Path, xrat_path: &str) -> String {
    render_service(DAEMON_SERVICE_TEMPLATE, exe, xrat_path)
}

#[cfg(target_os = "linux")]
fn generate_api_service(exe: &Path, xrat_path: &str) -> String {
    render_service(API_SERVICE_TEMPLATE, exe, xrat_path)
}

#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
fn systemctl_available() -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "status"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
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

// ---------------------------------------------------------------------------
// macOS: launchd user agents
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
const DAEMON_LABEL: &str = "com.xrat.daemon";
#[cfg(target_os = "macos")]
const API_LABEL: &str = "com.xrat.api";
#[cfg(target_os = "macos")]
const DAEMON_PLIST_NAME: &str = "com.xrat.daemon.plist";
#[cfg(target_os = "macos")]
const API_PLIST_NAME: &str = "com.xrat.api.plist";

#[cfg(target_os = "macos")]
const DAEMON_PLIST_TEMPLATE: &str =
    include_str!("../../../packaging/launchd/xrat-daemon.plist.template");
#[cfg(target_os = "macos")]
const API_PLIST_TEMPLATE: &str = include_str!("../../../packaging/launchd/xrat-api.plist.template");

#[cfg(target_os = "macos")]
fn launchd_agents_dir() -> crate::app::Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or(AppError::MissingHomeDirectory)?;
    Ok(PathBuf::from(home).join("Library").join("LaunchAgents"))
}

#[cfg(target_os = "macos")]
fn current_uid() -> crate::app::Result<String> {
    let output = std::process::Command::new("id")
        .arg("-u")
        .output()
        .map_err(|err| AppError::InvalidArgument(format!("failed to run `id -u`: {err}")))?;
    let uid = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if uid.is_empty() {
        return Err(AppError::InvalidArgument(
            "could not determine current uid".to_string(),
        ));
    }
    Ok(uid)
}

#[cfg(target_os = "macos")]
fn launchctl_available() -> bool {
    std::process::Command::new("launchctl")
        .arg("help")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

#[cfg(target_os = "macos")]
fn run_launchctl(args: &[&str]) -> std::io::Result<()> {
    let status = std::process::Command::new("launchctl")
        .args(args)
        .status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "launchctl {} exited with {}",
            args.join(" "),
            status
        )));
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn generate_daemon_plist(exe: &Path, xrat_path: &str) -> String {
    render_service(DAEMON_PLIST_TEMPLATE, exe, xrat_path)
}

#[cfg(target_os = "macos")]
fn generate_api_plist(exe: &Path, xrat_path: &str) -> String {
    render_service(API_PLIST_TEMPLATE, exe, xrat_path)
}

#[cfg(target_os = "macos")]
pub fn install(context: &AppContext, args: &DaemonInstallArgs) -> crate::app::Result<()> {
    if !args.dry_run && !launchctl_available() {
        return Err(AppError::InvalidArgument(
            "launchd is not available; this requires macOS with launchctl".to_string(),
        ));
    }

    let exe = resolve_exe();
    let xrat_path = context.runtime_paths.root_dir.display().to_string();
    let agents_dir = launchd_agents_dir()?;

    let daemon_content = generate_daemon_plist(&exe, &xrat_path);
    let api_content = generate_api_plist(&exe, &xrat_path);
    let daemon_path = agents_dir.join(DAEMON_PLIST_NAME);
    let api_path = agents_dir.join(API_PLIST_NAME);

    if args.dry_run {
        println!("--- dry run: no files written ---\n");
        println!("LaunchAgents directory: {}", agents_dir.display());
        println!();
        println!("--- {DAEMON_PLIST_NAME} ---");
        print!("{daemon_content}");
        if args.with_api {
            println!();
            println!("--- {API_PLIST_NAME} ---");
            print!("{api_content}");
        }
        println!();
        println!("Actions that would run:");
        println!(
            "  launchctl bootstrap gui/$(id -u) {}",
            daemon_path.display()
        );
        if args.with_api {
            println!("  launchctl bootstrap gui/$(id -u) {}", api_path.display());
        }
        if args.start {
            println!("  launchctl kickstart -k gui/$(id -u)/{DAEMON_LABEL}");
        }
        return Ok(());
    }

    std::fs::create_dir_all(&agents_dir)?;

    std::fs::write(&daemon_path, &daemon_content)?;
    println!("Written: {}", daemon_path.display());

    if args.with_api {
        std::fs::write(&api_path, &api_content)?;
        println!("Written: {}", api_path.display());
    }

    let uid = current_uid()?;
    let domain = format!("gui/{uid}");

    run_launchctl(&["bootstrap", &domain, &daemon_path.display().to_string()])?;
    println!("Bootstrapped: {DAEMON_LABEL}");

    if args.with_api {
        run_launchctl(&["bootstrap", &domain, &api_path.display().to_string()])?;
        println!("Bootstrapped: {API_LABEL}");
    }

    if args.start {
        run_launchctl(&["kickstart", "-k", &format!("{domain}/{DAEMON_LABEL}")])?;
        println!("Started: {DAEMON_LABEL}");
    }

    println!();
    println!("Daemon installed successfully.");
    if !args.start {
        println!("Start with: xrat daemon start");
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn uninstall(_context: &AppContext, args: &DaemonUninstallArgs) -> crate::app::Result<()> {
    let agents_dir = launchd_agents_dir()?;
    let daemon_path = agents_dir.join(DAEMON_PLIST_NAME);
    let api_path = agents_dir.join(API_PLIST_NAME);

    if args.dry_run {
        println!("--- dry run: no files removed ---\n");
        if daemon_path.exists() {
            println!("Would bootout: launchctl bootout gui/$(id -u)/{DAEMON_LABEL}");
            println!("Would remove:  {}", daemon_path.display());
        } else {
            println!("Not present: {}", daemon_path.display());
        }
        if api_path.exists() {
            println!("Would bootout: launchctl bootout gui/$(id -u)/{API_LABEL}");
            println!("Would remove:  {}", api_path.display());
        }
        return Ok(());
    }

    let uid = current_uid()?;
    let domain = format!("gui/{uid}");

    if daemon_path.exists() {
        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &format!("{domain}/{DAEMON_LABEL}")])
            .status();
        std::fs::remove_file(&daemon_path)?;
        println!("Removed: {}", daemon_path.display());
    } else {
        println!("Not present: {}", daemon_path.display());
    }

    if api_path.exists() {
        let _ = std::process::Command::new("launchctl")
            .args(["bootout", &format!("{domain}/{API_LABEL}")])
            .status();
        std::fs::remove_file(&api_path)?;
        println!("Removed: {}", api_path.display());
    }

    println!();
    println!("Daemon uninstalled. Config and data preserved.");

    Ok(())
}

// ---------------------------------------------------------------------------
// FreeBSD / OpenBSD: rc.d scripts
//
// rc.d scripts live in a system directory and the enable/start commands
// (sysrc/service on FreeBSD, rcctl on OpenBSD) require root. Run under sudo or
// as root.
// ---------------------------------------------------------------------------

#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
const DAEMON_RC_NAME: &str = "xrat_daemon";
#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
const API_RC_NAME: &str = "xrat_api";

#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
const DAEMON_RC_TEMPLATE: &str = include_str!("../../../packaging/rc.d/xrat_daemon.template");
#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
const API_RC_TEMPLATE: &str = include_str!("../../../packaging/rc.d/xrat_api.template");

#[cfg(target_os = "freebsd")]
fn rc_d_dir() -> PathBuf {
    PathBuf::from("/usr/local/etc/rc.d")
}
#[cfg(target_os = "openbsd")]
fn rc_d_dir() -> PathBuf {
    PathBuf::from("/etc/rc.d")
}

#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
fn generate_daemon_rc(exe: &Path, xrat_path: &str) -> String {
    render_service(DAEMON_RC_TEMPLATE, exe, xrat_path)
}
#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
fn generate_api_rc(exe: &Path, xrat_path: &str) -> String {
    render_service(API_RC_TEMPLATE, exe, xrat_path)
}

#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
fn run_status(cmd: &str, args: &[&str]) -> std::io::Result<()> {
    let status = std::process::Command::new(cmd).args(args).status()?;
    if !status.success() {
        return Err(std::io::Error::other(format!(
            "{cmd} {} exited with {}",
            args.join(" "),
            status
        )));
    }
    Ok(())
}

#[cfg(target_os = "freebsd")]
fn enable_service(rc_name: &str) -> std::io::Result<()> {
    run_status("sysrc", &[&format!("{rc_name}_enable=YES")])
}
#[cfg(target_os = "freebsd")]
fn start_service(rc_name: &str) -> std::io::Result<()> {
    run_status("service", &[rc_name, "start"])
}
#[cfg(target_os = "freebsd")]
fn stop_and_disable_service(rc_name: &str) {
    let _ = std::process::Command::new("service")
        .args([rc_name, "stop"])
        .status();
    let _ = std::process::Command::new("sysrc")
        .arg(format!("{rc_name}_enable=NO"))
        .status();
}

#[cfg(target_os = "openbsd")]
fn enable_service(rc_name: &str) -> std::io::Result<()> {
    run_status("rcctl", &["enable", rc_name])
}
#[cfg(target_os = "openbsd")]
fn start_service(rc_name: &str) -> std::io::Result<()> {
    run_status("rcctl", &["start", rc_name])
}
#[cfg(target_os = "openbsd")]
fn stop_and_disable_service(rc_name: &str) {
    let _ = std::process::Command::new("rcctl")
        .args(["stop", rc_name])
        .status();
    let _ = std::process::Command::new("rcctl")
        .args(["disable", rc_name])
        .status();
}

#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
fn write_rc_script(path: &Path, content: &str) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(path, content)?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
pub fn install(context: &AppContext, args: &DaemonInstallArgs) -> crate::app::Result<()> {
    let exe = resolve_exe();
    let xrat_path = context.runtime_paths.root_dir.display().to_string();
    let dir = rc_d_dir();

    let daemon_content = generate_daemon_rc(&exe, &xrat_path);
    let api_content = generate_api_rc(&exe, &xrat_path);
    let daemon_path = dir.join(DAEMON_RC_NAME);
    let api_path = dir.join(API_RC_NAME);

    if args.dry_run {
        println!("--- dry run: no files written ---\n");
        println!("rc.d directory: {}", dir.display());
        println!();
        println!("--- {DAEMON_RC_NAME} ---");
        print!("{daemon_content}");
        if args.with_api {
            println!();
            println!("--- {API_RC_NAME} ---");
            print!("{api_content}");
        }
        println!();
        println!("Actions that would run (require root):");
        println!("  enable service {DAEMON_RC_NAME}");
        if args.with_api {
            println!("  enable service {API_RC_NAME}");
        }
        if args.start {
            println!("  start service {DAEMON_RC_NAME}");
        }
        return Ok(());
    }

    std::fs::create_dir_all(&dir)?;

    write_rc_script(&daemon_path, &daemon_content)?;
    println!("Written: {}", daemon_path.display());

    if args.with_api {
        write_rc_script(&api_path, &api_content)?;
        println!("Written: {}", api_path.display());
    }

    enable_service(DAEMON_RC_NAME)?;
    println!("Enabled: {DAEMON_RC_NAME}");

    if args.with_api {
        enable_service(API_RC_NAME)?;
        println!("Enabled: {API_RC_NAME}");
    }

    if args.start {
        start_service(DAEMON_RC_NAME)?;
        println!("Started: {DAEMON_RC_NAME}");
    }

    println!();
    println!("Daemon installed successfully.");
    if !args.start {
        println!("Start with: xrat daemon start");
    }

    Ok(())
}

#[cfg(any(target_os = "freebsd", target_os = "openbsd"))]
pub fn uninstall(_context: &AppContext, args: &DaemonUninstallArgs) -> crate::app::Result<()> {
    let dir = rc_d_dir();
    let daemon_path = dir.join(DAEMON_RC_NAME);
    let api_path = dir.join(API_RC_NAME);

    if args.dry_run {
        println!("--- dry run: no files removed ---\n");
        if daemon_path.exists() {
            println!("Would stop and disable service {DAEMON_RC_NAME}");
            println!("Would remove: {}", daemon_path.display());
        } else {
            println!("Not present: {}", daemon_path.display());
        }
        if api_path.exists() {
            println!("Would stop and disable service {API_RC_NAME}");
            println!("Would remove: {}", api_path.display());
        }
        return Ok(());
    }

    if daemon_path.exists() {
        stop_and_disable_service(DAEMON_RC_NAME);
        std::fs::remove_file(&daemon_path)?;
        println!("Removed: {}", daemon_path.display());
    } else {
        println!("Not present: {}", daemon_path.display());
    }

    if api_path.exists() {
        stop_and_disable_service(API_RC_NAME);
        std::fs::remove_file(&api_path)?;
        println!("Removed: {}", api_path.display());
    }

    println!();
    println!("Daemon uninstalled. Config and data preserved.");

    Ok(())
}

// ---------------------------------------------------------------------------
// Unsupported platforms
// ---------------------------------------------------------------------------

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd"
)))]
pub fn install(_context: &AppContext, _args: &DaemonInstallArgs) -> crate::app::Result<()> {
    Err(AppError::UnsupportedPlatform(
        "daemon install is not supported on this platform".to_string(),
    ))
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd"
)))]
pub fn uninstall(_context: &AppContext, _args: &DaemonUninstallArgs) -> crate::app::Result<()> {
    Err(AppError::UnsupportedPlatform(
        "daemon uninstall is not supported on this platform".to_string(),
    ))
}
