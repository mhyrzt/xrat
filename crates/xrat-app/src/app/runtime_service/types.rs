pub(super) use std::path::PathBuf;
pub(super) use std::time::Duration;

pub(super) use tokio::time::timeout;

pub(super) use crate::app::AppError;
pub(super) use crate::app::config::defaults;
pub(super) use crate::app::context::AppContext;
pub(super) use crate::app::services::runtime_tuning::{
    apply_xray_dns_options, apply_xray_routing_options, build_singbox_dns_options,
    build_singbox_routing_options, build_xray_gen_options, ensure_xray_tun_supported_with_spawner,
    resolve_listen_interface_addr,
};
pub(super) use xrat_db::{
    ConfigListFilter, ConfigRecord, RuntimeSessionInsert, RuntimeSessionRecord,
    RuntimeSessionStatus,
};
pub(super) use xrat_engines::singbox::{
    SingboxClashApi, SingboxConfig, SingboxInbound, SingboxInboundUser, SingboxTunOptions,
    generate_singbox_runtime_config_with_dns, process_mgmt as singbox_runtime,
};
pub(super) use xrat_engines::xray::config::{
    Inbound, XrayTunCaptureOptions, enable_stats_api, enable_tun_capture, enable_tun_split_routing,
};
pub(super) use xrat_engines::xray::{
    generate_runtime_config_for_inbounds_with_options, runtime_process as xray_runtime,
};
pub(super) use xrat_model::{ConfigId, Protocol};

pub(super) use xrat_support::time::now_string;

pub(super) const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
pub(super) const INBOUND_LIVENESS_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectRequest {
    pub config_id: ConfigId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceRequest {
    pub trigger: crate::app::services::rotation::RotationTrigger,
    pub candidate_id: Option<ConfigId>,
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
    pub old_session_id: Option<i64>,
    pub new_config_id: ConfigId,
    pub new_session_id: i64,
    pub new_pid: u32,
}

#[derive(Clone, Debug)]
pub struct RuntimeStatusSnapshot {
    pub status: RuntimeSessionDisplay,
    pub session: Option<RuntimeSessionRecord>,
    pub session_config: Option<ConfigRecord>,
    pub active_config: Option<ConfigRecord>,
    pub pid_running: bool,
    pub inbound_health: RuntimeInboundHealth,
    pub database_label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeSessionDisplay {
    Degraded,
    Persisted(RuntimeSessionStatus),
    Stale,
    StaleReconciled,
    Stopped,
}

impl RuntimeSessionDisplay {
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
    pub(super) rollback_context: Option<&'a AppContext>,
    pub(super) process_ports: xrat_support::readiness::RuntimeProcessPorts,
}
