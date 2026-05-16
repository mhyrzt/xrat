use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use tempfile::NamedTempFile;
use tokio::net::TcpStream;
use tokio::time::sleep;

use super::super::config::XrayConfig;
use super::errors::XrayProcessError;

pub struct XrayProcess {
    child: Child,
    config_file: NamedTempFile,
    local_port: u16,
}

impl XrayProcess {
    pub async fn spawn(
        config: &XrayConfig,
        startup_timeout: Duration,
    ) -> Result<Self, XrayProcessError> {
        Self::spawn_with_binary(Path::new("xray"), config, startup_timeout).await
    }

    pub async fn spawn_with_binary(
        binary_path: &Path,
        config: &XrayConfig,
        startup_timeout: Duration,
    ) -> Result<Self, XrayProcessError> {
        let mut temp_file = NamedTempFile::new()?;
        let config_json = serde_json::to_string_pretty(config)?;
        temp_file.write_all(config_json.as_bytes())?;
        temp_file.flush()?;

        let config_path = temp_file.path().to_path_buf();
        let local_port = config
            .inbounds
            .first()
            .map(|inbound| inbound.port)
            .unwrap_or(0);

        let child = Command::new(binary_path)
            .arg("run")
            .arg("-c")
            .arg(&config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| XrayProcessError::SpawnError(error.to_string()))?;

        let mut process = XrayProcess {
            child,
            config_file: temp_file,
            local_port,
        };
        process.wait_for_ready(startup_timeout).await?;
        Ok(process)
    }

    async fn wait_for_ready(&mut self, timeout: Duration) -> Result<(), XrayProcessError> {
        let start = Instant::now();
        let check_interval = Duration::from_millis(100);

        loop {
            if let Ok(Some(_)) = self.child.try_wait() {
                return Err(XrayProcessError::ProcessExited(self.read_stderr()));
            }
            if TcpStream::connect(format!("127.0.0.1:{}", self.local_port))
                .await
                .is_ok()
            {
                return Ok(());
            }
            if start.elapsed() >= timeout {
                let _ = self.child.kill();
                return Err(XrayProcessError::PortNotReady(self.local_port));
            }
            sleep(check_interval).await;
        }
    }

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_file.path().to_path_buf()
    }

    pub fn kill(mut self) -> Result<(), std::io::Error> {
        self.child.kill()?;
        let _ = self.child.wait();
        Ok(())
    }

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
