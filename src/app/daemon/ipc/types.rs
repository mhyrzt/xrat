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
    ProxyStart,
    ProxyStatus,
    ProxyStop,
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
    pub http_api_enabled: bool,
    pub http_api_addr: Option<String>,
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
    pub old_session_id: Option<i64>,
    pub new_config_id: i64,
    pub new_session_id: i64,
    pub new_pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyControlPayload {
    pub rotation_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyStatusPayload {
    pub daemon_ready: bool,
    pub rotation_enabled: bool,
    pub interval_secs: u64,
    pub health_trigger_enabled: bool,
    pub cooldown_secs: u64,
    pub active_config_id: Option<i64>,
    pub last_trigger: Option<RotationTrigger>,
    pub last_result: String,
    pub last_candidate_config_id: Option<i64>,
    pub last_candidate_result: String,
    pub cooldown_active: bool,
    pub next_timer_epoch_secs: Option<u64>,
    pub health_failure_threshold: u32,
    pub consecutive_health_failures: u32,
    pub health_probe_in_flight: bool,
    pub last_health_check_epoch_secs: Option<u64>,
    pub last_health_error: Option<String>,
    pub pending_health_recovery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonShutdownPayload {
    pub daemon_ready: bool,
    pub runtime_disconnected: bool,
}
