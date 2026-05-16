pub(super) use std::path::PathBuf;
pub(super) use std::time::Duration;

pub(super) use tokio::net::TcpStream;
pub(super) use tokio::time::timeout;

pub(super) use crate::app::AppError;
pub(super) use crate::app::config::defaults;
pub(super) use crate::app::context::AppContext;
pub(super) use crate::db::{
    ConfigListFilter, ConfigRecord, RuntimeSessionInsert, RuntimeSessionRecord,
    RuntimeSessionStatus,
};
pub(super) use crate::xray::config::Inbound;
pub(super) use crate::xray::{generate_runtime_config_for_inbounds, process_mgmt as xray_runtime};

pub(super) use crate::support::time::now_string;

pub(super) const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const INBOUND_LIVENESS_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectRequest {
    pub config_id: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceRequest {
    pub trigger: crate::app::daemon::ipc::RotationTrigger,
    pub candidate_id: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct ConnectResult {
    pub config: ConfigRecord,
    pub session_id: i64,
    pub pid: u32,
    pub runtime_config_path: PathBuf,
    pub endpoints: RuntimeEndpoints,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisconnectResult {
    pub stopped_session: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceResult {
    pub old_session_id: i64,
    pub new_config_id: i64,
    pub new_session_id: i64,
    pub new_pid: u32,
}

#[derive(Clone, Debug)]
pub struct RuntimeStatusSnapshot {
    pub status: RuntimeStatusLabel,
    pub session: Option<RuntimeSessionRecord>,
    pub session_config: Option<ConfigRecord>,
    pub active_config: Option<ConfigRecord>,
    pub selected_config: Option<ConfigRecord>,
    pub pid_running: bool,
    pub inbound_health: RuntimeInboundHealth,
    pub database_label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeStatusLabel {
    Degraded,
    Persisted(RuntimeSessionStatus),
    Stale,
    StaleReconciled,
    Stopped,
}

impl RuntimeStatusLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Degraded => "degraded",
            Self::Persisted(status) => status.as_str(),
            Self::Stale => "stale",
            Self::StaleReconciled => "stale reconciled",
            Self::Stopped => "stopped",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeEndpoints {
    pub socks: Option<RuntimeEndpoint>,
    pub http: Option<RuntimeEndpoint>,
    pub shadowsocks: Option<RuntimeEndpoint>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeEndpoint {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeInboundHealth {
    pub socks: Option<RuntimeEndpointHealth>,
    pub http: Option<RuntimeEndpointHealth>,
    pub shadowsocks: Option<RuntimeEndpointHealth>,
}

impl RuntimeInboundHealth {
    pub(crate) fn has_unreachable_endpoint(&self) -> bool {
        [&self.socks, &self.http, &self.shadowsocks]
            .into_iter()
            .flatten()
            .any(|health| matches!(health.state, RuntimeEndpointState::Unreachable))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeEndpointHealth {
    pub endpoint: RuntimeEndpoint,
    pub state: RuntimeEndpointState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeEndpointState {
    Reachable,
    Unreachable,
    NotChecked,
}

impl RuntimeEndpointState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reachable => "reachable",
            Self::Unreachable => "unreachable",
            Self::NotChecked => "not checked",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActiveSessionState {
    None,
    Running(RuntimeSessionRecord),
    Stale(RuntimeSessionRecord),
}

pub struct RuntimeService<'a> {
    pub(super) context: &'a AppContext,
}
