use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::net::TcpStream;
use tokio::time::sleep;

use crate::singbox::SingboxConfig;

#[derive(Debug, Error)]
pub enum SingboxRuntimeError {
    #[error("failed to create runtime files: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize sing-box config: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to spawn sing-box process: {0}")]
    Spawn(String),
    #[error("sing-box process exited during startup: {0}")]
    ProcessExited(String),
    #[error("sing-box did not open local port {port} before startup timeout")]
    StartupTimeout { port: u16 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedSingboxPaths {
    pub config_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManagedSingboxProcess {
    pub pid: u32,
    pub ready_port: u16,
    pub paths: ManagedSingboxPaths,
}

pub async fn spawn_detached(
    binary_path: &Path,
    runtime_dir: &Path,
    session_id: i64,
    config: &SingboxConfig,
    ready_host: &str,
    ready_port: u16,
    startup_timeout: Duration,
) -> Result<ManagedSingboxProcess, SingboxRuntimeError> {
    std::fs::create_dir_all(runtime_dir)?;

    let paths = ManagedSingboxPaths {
        config_path: runtime_dir.join(format!("session-{session_id}.singbox.json")),
        stdout_path: runtime_dir.join(format!("session-{session_id}.singbox.out.log")),
        stderr_path: runtime_dir.join(format!("session-{session_id}.singbox.err.log")),
    };

    let mut config_file = File::create(&paths.config_path)?;
    config_file.write_all(serde_json::to_string_pretty(config)?.as_bytes())?;
    config_file.flush()?;

    let stdout = File::create(&paths.stdout_path)?;
    let stderr = File::create(&paths.stderr_path)?;
    let mut child = Command::new(binary_path)
        .arg("run")
        .arg("-c")
        .arg(&paths.config_path)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|error| SingboxRuntimeError::Spawn(error.to_string()))?;

    let pid = child.id();
    match wait_for_ready(&mut child, ready_host, ready_port, startup_timeout).await {
        Ok(()) => Ok(ManagedSingboxProcess {
            pid,
            ready_port,
            paths,
        }),
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(error)
        }
    }
}

async fn wait_for_ready(
    child: &mut std::process::Child,
    host: &str,
    port: u16,
    timeout: Duration,
) -> Result<(), SingboxRuntimeError> {
    let start = Instant::now();
    let check_interval = Duration::from_millis(100);
    let address = format!("{host}:{port}");

    loop {
        if let Some(status) = child.try_wait()? {
            return Err(SingboxRuntimeError::ProcessExited(status.to_string()));
        }

        if TcpStream::connect(&address).await.is_ok() {
            return Ok(());
        }

        if start.elapsed() >= timeout {
            return Err(SingboxRuntimeError::StartupTimeout { port });
        }

        sleep(check_interval).await;
    }
}
