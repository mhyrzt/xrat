use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::time::sleep;

use super::config::XrayConfig;

#[derive(Debug, Error)]
pub enum XrayProcessError {
    #[error("Failed to create temp config file: {0}")]
    TempFileError(#[from] std::io::Error),

    #[error("Failed to serialize config: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Failed to spawn xray process: {0}")]
    SpawnError(String),

    #[error("Xray process failed to start within timeout")]
    StartupTimeout,

    #[error("Xray process exited unexpectedly: {0}")]
    ProcessExited(String),

    #[error("Port {0} not ready within timeout")]
    PortNotReady(u16),
}

pub struct XrayProcess {
    child: Child,
    config_file: NamedTempFile,
    local_port: u16,
}

impl XrayProcess {
    /// Spawn a new Xray process with the given config
    pub async fn spawn(
        config: &XrayConfig,
        startup_timeout: Duration,
    ) -> Result<Self, XrayProcessError> {
        Self::spawn_with_binary(Path::new("xray"), config, startup_timeout).await
    }

    /// Spawn a new Xray-compatible process with an explicit binary path.
    pub async fn spawn_with_binary(
        binary_path: &Path,
        config: &XrayConfig,
        startup_timeout: Duration,
    ) -> Result<Self, XrayProcessError> {
        // Create temp config file
        let mut temp_file = NamedTempFile::new()?;
        let config_json = serde_json::to_string_pretty(config)?;
        temp_file.write_all(config_json.as_bytes())?;
        temp_file.flush()?;

        let config_path = temp_file.path().to_path_buf();
        let local_port = config.inbounds.first().map(|i| i.port).unwrap_or(0);

        // Spawn xray process
        let child = Command::new(binary_path)
            .arg("run")
            .arg("-c")
            .arg(&config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| XrayProcessError::SpawnError(e.to_string()))?;

        let mut process = XrayProcess {
            child,
            config_file: temp_file,
            local_port,
        };

        // Wait for the process to be ready
        process.wait_for_ready(startup_timeout).await?;

        Ok(process)
    }

    /// Wait for the local port to be ready
    async fn wait_for_ready(&mut self, timeout: Duration) -> Result<(), XrayProcessError> {
        let start = Instant::now();
        let check_interval = Duration::from_millis(100);

        loop {
            // Check if process has exited
            if let Ok(Some(_)) = self.child.try_wait() {
                return Err(XrayProcessError::ProcessExited(self.read_stderr()));
            }

            // Try to connect to the local port
            if TcpStream::connect(format!("127.0.0.1:{}", self.local_port))
                .await
                .is_ok()
            {
                return Ok(());
            }

            // Check timeout
            if start.elapsed() >= timeout {
                let _ = self.child.kill();
                return Err(XrayProcessError::PortNotReady(self.local_port));
            }

            sleep(check_interval).await;
        }
    }

    /// Get the process ID
    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Get the local port
    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    /// Get the config file path
    pub fn config_path(&self) -> PathBuf {
        self.config_file.path().to_path_buf()
    }

    /// Kill the process and clean up
    pub fn kill(mut self) -> Result<(), std::io::Error> {
        self.child.kill()?;
        let _ = self.child.wait();
        Ok(())
    }

    /// Wait for the process to exit
    pub fn wait(mut self) -> Result<std::process::ExitStatus, std::io::Error> {
        self.child.wait()
    }

    fn read_stderr(&mut self) -> String {
        let Some(stderr) = self.child.stderr.as_mut() else {
            return "stderr unavailable".to_string();
        };

        let mut output = String::new();
        match stderr.read_to_string(&mut output) {
            Ok(_) if !output.trim().is_empty() => output.trim().to_string(),
            Ok(_) => "process exited without stderr output".to_string(),
            Err(error) => format!("failed to read stderr: {error}"),
        }
    }
}

impl Drop for XrayProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests;
