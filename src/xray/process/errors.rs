use thiserror::Error;

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
