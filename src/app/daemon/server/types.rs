use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonRequest {
    pub protocol_version: u16,
    pub request: DaemonRequestKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DaemonRequestKind {
    DaemonPing,
    DaemonShutdown,
    RuntimeStatus,
    RuntimeConnect {
        config_id: i64,
    },
    RuntimeReplace {
        trigger: RotationTrigger,
        candidate_id: Option<i64>,
    },
    RuntimeDisconnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RotationTrigger {
    Manual,
    Timer,
    HealthCheckFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonResponse<T> {
    pub protocol_version: u16,
    pub ok: bool,
    pub code: DaemonResponseCode,
    pub message: String,
    pub payload: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonResponseCode {
    Ok,
    Busy,
    NotFound,
    InvalidState,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingPayload {
    pub daemon_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatusPayload {
    pub daemon_ready: bool,
    pub runtime_owned: bool,
    pub runtime_status: String,
    pub session_id: Option<i64>,
    pub active_config_id: Option<i64>,
    pub pid_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConnectPayload {
    pub config_id: i64,
    pub session_id: i64,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDisconnectPayload {
    pub stopped_session: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReplacePayload {
    pub trigger: RotationTrigger,
    pub replaced: bool,
    pub old_session_id: i64,
    pub new_config_id: i64,
    pub new_session_id: i64,
    pub new_pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonShutdownPayload {
    pub daemon_ready: bool,
    pub runtime_disconnected: bool,
}
