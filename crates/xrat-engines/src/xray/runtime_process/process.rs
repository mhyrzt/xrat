use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;
use xrat_support::process::{Command, StartupChild, Stdio};

use thiserror::Error;

use crate::xray::XrayConfig;

#[derive(Debug, Error)]
pub enum XrayRuntimeError {
    #[error("failed to prepare Xray runtime files: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize Xray runtime config: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to spawn Xray process: {0}")]
    Spawn(String),
    #[error("Xray process exited during startup: {0}")]
    ProcessExited(String),
    #[error("Xray did not open local port {port} before startup timeout")]
    StartupTimeout { port: u16 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedXrayPaths {
    pub config_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedXrayProcess {
    pub pid: u32,
    pub ready_port: u16,
    pub paths: ManagedXrayPaths,
}

pub async fn spawn_detached(
    binary_path: &Path,
    runtime_dir: &Path,
    session_id: i64,
    config: &XrayConfig,
    ready_host: &str,
    ready_port: u16,
    startup_timeout: Duration,
) -> Result<ManagedXrayProcess, XrayRuntimeError> {
    spawn_detached_with_ports(
        binary_path,
        runtime_dir,
        session_id,
        config,
        ready_host,
        ready_port,
        startup_timeout,
        xrat_support::readiness::RuntimeProcessPorts::default(),
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn spawn_detached_with_ports(
    binary_path: &Path,
    runtime_dir: &Path,
    session_id: i64,
    config: &XrayConfig,
    ready_host: &str,
    ready_port: u16,
    startup_timeout: Duration,
    ports: xrat_support::readiness::RuntimeProcessPorts,
) -> Result<ManagedXrayProcess, XrayRuntimeError> {
    spawn_detached_json_with_ports(
        binary_path,
        runtime_dir,
        session_id,
        serde_json::to_string_pretty(config)?.as_bytes(),
        ready_host,
        ready_port,
        startup_timeout,
        ports,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn spawn_detached_json_with_ports(
    binary_path: &Path,
    runtime_dir: &Path,
    session_id: i64,
    config: &[u8],
    ready_host: &str,
    ready_port: u16,
    startup_timeout: Duration,
    ports: xrat_support::readiness::RuntimeProcessPorts,
) -> Result<ManagedXrayProcess, XrayRuntimeError> {
    std::fs::create_dir_all(runtime_dir)?;

    let paths = ManagedXrayPaths {
        config_path: runtime_dir.join(format!("session-{session_id}.json")),
        stdout_path: runtime_dir.join(format!("session-{session_id}.out.log")),
        stderr_path: runtime_dir.join(format!("session-{session_id}.err.log")),
    };

    let mut config_file = tempfile::NamedTempFile::new_in(runtime_dir)?;
    config_file.write_all(config)?;
    config_file.flush()?;
    config_file
        .persist(&paths.config_path)
        .map_err(|error| error.error)?;

    let stdout = tempfile::NamedTempFile::new_in(runtime_dir)?
        .persist(&paths.stdout_path)
        .map_err(|error| error.error)?;
    let stderr = tempfile::NamedTempFile::new_in(runtime_dir)?
        .persist(&paths.stderr_path)
        .map_err(|error| error.error)?;
    let mut command = Command::with_spawner(binary_path, ports.spawner.clone());
    configure_asset_path(&mut command, binary_path);
    let child = command
        .arg("run")
        .arg("-c")
        .arg(&paths.config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .detached(true)
        .spawn()
        .map_err(|error| XrayRuntimeError::Spawn(error.to_string()))?;

    let pid = child.id();
    let mut startup = StartupChild::new(child);
    match wait_for_ready(
        startup.child_mut(),
        ready_host,
        ready_port,
        startup_timeout,
        ports.waiter.as_ref(),
    )
    .await
    {
        Ok(()) => {
            startup.detach();
            Ok(ManagedXrayProcess {
                pid,
                ready_port,
                paths,
            })
        }
        Err(error) => {
            drop(startup);
            Err(error.with_process_stderr(&paths.stderr_path))
        }
    }
}

fn configure_asset_path(command: &mut Command, binary_path: &Path) {
    let Some(directory) = xrat_support::platform::managed_core_asset_dir(binary_path) else {
        return;
    };
    let variable = if binary_path.file_name().and_then(|name| name.to_str()) == Some("v2ray") {
        "V2RAY_LOCATION_ASSET"
    } else {
        "XRAY_LOCATION_ASSET"
    };
    command.env(variable, directory);
}

async fn wait_for_ready(
    child: &mut xrat_support::process::Child,
    host: &str,
    port: u16,
    timeout: Duration,
    waiter: &dyn xrat_support::readiness::PortWaiter,
) -> Result<(), XrayRuntimeError> {
    use xrat_support::readiness::{ReadinessError, ReadinessRequest};
    waiter
        .wait(child, ReadinessRequest::single(host, port, timeout))
        .await
        .map_err(|error| match error {
            ReadinessError::Io(error) => XrayRuntimeError::Io(error),
            ReadinessError::ProcessExited(status) => {
                XrayRuntimeError::ProcessExited(status.to_string())
            }
            ReadinessError::Timeout { port } => XrayRuntimeError::StartupTimeout { port },
        })
}

pub fn process_is_running(pid: i64) -> bool {
    xrat_support::signals::ProcessSignals::is_running(
        &xrat_support::signals::SystemProcessSignals::default(),
        pid,
    )
}

trait StartupErrorExt {
    fn with_process_stderr(self, stderr_path: &Path) -> Self;
}

impl StartupErrorExt for XrayRuntimeError {
    fn with_process_stderr(self, stderr_path: &Path) -> Self {
        let Some(stderr_tail) = read_stderr_tail(stderr_path) else {
            return self;
        };

        match self {
            XrayRuntimeError::ProcessExited(status) => {
                XrayRuntimeError::ProcessExited(format!("{status}; stderr: {stderr_tail}"))
            }
            XrayRuntimeError::StartupTimeout { port } => XrayRuntimeError::Spawn(format!(
                "Xray did not open local port {port} before startup timeout; stderr: {stderr_tail}"
            )),
            other => other,
        }
    }
}

fn read_stderr_tail(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut output = String::new();
    file.read_to_string(&mut output).ok()?;
    let output = output.trim();
    if output.is_empty() {
        return None;
    }

    let mut lines: Vec<&str> = output.lines().rev().take(6).collect();
    lines.reverse();
    Some(lines.join(" | "))
}
