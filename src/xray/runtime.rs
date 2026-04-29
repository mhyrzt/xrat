use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use tokio::net::TcpStream;
use tokio::time::sleep;

use crate::xray::XrayConfig;

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
) -> Result<ManagedXrayProcess, crate::app::AppError> {
    std::fs::create_dir_all(runtime_dir)?;

    let paths = ManagedXrayPaths {
        config_path: runtime_dir.join(format!("session-{session_id}.json")),
        stdout_path: runtime_dir.join(format!("session-{session_id}.out.log")),
        stderr_path: runtime_dir.join(format!("session-{session_id}.err.log")),
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
        .map_err(|error| crate::app::AppError::XraySpawn(error.to_string()))?;

    let pid = child.id();
    match wait_for_ready(&mut child, ready_host, ready_port, startup_timeout).await {
        Ok(()) => Ok(ManagedXrayProcess {
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
) -> Result<(), crate::app::AppError> {
    let start = Instant::now();
    let check_interval = Duration::from_millis(100);
    let address = format!("{host}:{port}");

    loop {
        if let Some(status) = child.try_wait()? {
            return Err(crate::app::AppError::XrayExited(status.to_string()));
        }

        if TcpStream::connect(&address).await.is_ok() {
            return Ok(());
        }

        if start.elapsed() >= timeout {
            return Err(crate::app::AppError::XrayStartupTimeout { port });
        }

        sleep(check_interval).await;
    }
}

pub fn process_is_running(pid: i64) -> bool {
    if pid <= 0 {
        return false;
    }

    #[cfg(unix)]
    {
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

pub fn terminate_process(pid: i64) -> Result<bool, crate::app::AppError> {
    if pid <= 0 {
        return Ok(false);
    }

    #[cfg(unix)]
    {
        let status = Command::new("kill").arg(pid.to_string()).status()?;
        Ok(status.success())
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::process_is_running;

    #[test]
    fn invalid_pid_is_not_running() {
        assert!(!process_is_running(0));
        assert!(!process_is_running(-1));
    }
}
