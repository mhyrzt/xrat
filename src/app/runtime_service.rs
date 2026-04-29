use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::app::AppError;
use crate::app::config::defaults;
use crate::app::runtime::AppContext;
use crate::db::{ConfigRecord, RuntimeSessionInsert, RuntimeSessionRecord, RuntimeSessionStatus};
use crate::model::{Node, Protocol};
use crate::xray::config::Inbound;
use crate::xray::{generate_runtime_config_for_inbounds, runtime as xray_runtime};

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);
const INBOUND_LIVENESS_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectRequest {
    pub config_id: i64,
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
    fn has_unreachable_endpoint(&self) -> bool {
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
    context: &'a AppContext,
}

impl<'a> RuntimeService<'a> {
    pub fn new(context: &'a AppContext) -> Self {
        Self { context }
    }

    pub async fn connect(&self, request: ConnectRequest) -> crate::app::Result<ConnectResult> {
        let Some(config) = self.context.db.get_config_by_id(request.config_id).await? else {
            return Err(AppError::InvalidArgument(format!(
                "config {} was not found",
                request.config_id
            )));
        };
        if !config.is_enabled {
            return Err(AppError::InvalidArgument(format!(
                "config {} is disabled",
                request.config_id
            )));
        }

        match self.active_session_state().await? {
            ActiveSessionState::Running(session) => {
                if !self.context.app_config.runtime.replace_active_session {
                    tracing::warn!(
                        session_id = session.id,
                        "active runtime session blocks connect"
                    );
                    return Err(AppError::RuntimeSessionAlreadyActive);
                }

                self.disconnect().await?;
            }
            ActiveSessionState::Stale(session) => {
                tracing::warn!(
                    session_id = session.id,
                    "stale runtime session was reconciled before connect"
                );
            }
            ActiveSessionState::None => {}
        }

        let launch = self.resolve_launch(&config)?;
        let session_id = self
            .context
            .db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(config.id),
                status: RuntimeSessionStatus::Starting,
                socks_host: launch
                    .endpoints
                    .socks
                    .as_ref()
                    .map(|inbound| inbound.host.clone()),
                socks_port: launch
                    .endpoints
                    .socks
                    .as_ref()
                    .map(|inbound| i64::from(inbound.port)),
                http_host: launch
                    .endpoints
                    .http
                    .as_ref()
                    .map(|inbound| inbound.host.clone()),
                http_port: launch
                    .endpoints
                    .http
                    .as_ref()
                    .map(|inbound| i64::from(inbound.port)),
                shadowsocks_host: launch
                    .endpoints
                    .shadowsocks
                    .as_ref()
                    .map(|inbound| inbound.host.clone()),
                shadowsocks_port: launch
                    .endpoints
                    .shadowsocks
                    .as_ref()
                    .map(|inbound| i64::from(inbound.port)),
                process_id: None,
                failure_reason: None,
                started_at: None,
                stopped_at: None,
            })
            .await?;

        let process = match xray_runtime::spawn_detached(
            &launch.binary_path,
            &self.context.runtime_paths.runtime_dir,
            session_id,
            &launch.config,
            &launch.ready_host,
            launch.ready_port,
            Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
        )
        .await
        {
            Ok(process) => process,
            Err(error) => {
                self.context
                    .db
                    .update_runtime_session_state(
                        session_id,
                        RuntimeSessionStatus::Failed,
                        None,
                        None,
                        Some(&now_string()),
                        Some(&error.to_string()),
                    )
                    .await?;
                return Err(error);
            }
        };

        self.context
            .db
            .update_runtime_session_state(
                session_id,
                RuntimeSessionStatus::Running,
                Some(i64::from(process.pid)),
                Some(&now_string()),
                None,
                None,
            )
            .await?;
        self.context.db.set_active_config(config.id).await?;

        Ok(ConnectResult {
            config,
            session_id,
            pid: process.pid,
            runtime_config_path: process.paths.config_path,
            endpoints: launch.endpoints,
        })
    }

    pub async fn disconnect(&self) -> crate::app::Result<DisconnectResult> {
        let stopped_session = stop_active_session(self.context).await?;
        Ok(DisconnectResult { stopped_session })
    }

    pub async fn status(&self) -> crate::app::Result<RuntimeStatusSnapshot> {
        let active_state = self.active_session_state().await?;
        let latest = self.context.db.get_latest_runtime_session().await?;
        let session_config = match latest.as_ref().and_then(|session| session.config_id) {
            Some(config_id) => self.context.db.get_config_by_id(config_id).await?,
            None => None,
        };
        let active_config = self.context.db.get_active_config().await?;
        let selected_config = self.context.db.get_selected_config().await?;
        let pid_running = latest.as_ref().is_some_and(runtime_session_is_alive);
        let inbound_health = match &latest {
            Some(session) => check_runtime_inbounds(session, pid_running).await,
            None => RuntimeInboundHealth::default(),
        };
        let status = runtime_status_label(&latest, &active_state, pid_running, &inbound_health);

        Ok(RuntimeStatusSnapshot {
            status,
            session: latest,
            session_config,
            active_config,
            selected_config,
            pid_running,
            inbound_health,
            database_label: self.context.runtime_paths.database_label.clone(),
        })
    }

    pub async fn active_session_state(&self) -> crate::app::Result<ActiveSessionState> {
        active_session_state(self.context).await
    }

    fn resolve_launch(&self, config: &ConfigRecord) -> crate::app::Result<ResolvedLaunch> {
        let runtime = &self.context.app_config.runtime;
        let socks = runtime.socks.enabled.then_some((
            runtime.socks.host.as_str(),
            runtime.socks.port,
            runtime.socks.udp,
        ));
        let http = runtime
            .http
            .enabled
            .then_some((runtime.http.host.as_str(), runtime.http.port));
        let shadowsocks = if runtime.shadowsocks.enabled {
            Some((
                runtime.shadowsocks.host.as_str(),
                runtime.shadowsocks.port,
                runtime.shadowsocks.method.as_str(),
                runtime.shadowsocks.password.resolve()?,
                runtime.shadowsocks.network.as_str(),
            ))
        } else {
            None
        };

        if socks.is_none() && http.is_none() && shadowsocks.is_none() {
            return Err(AppError::NoRuntimeInboundEnabled);
        }

        let node = node_from_record(config)?;
        let mut xray_config = generate_runtime_config_for_inbounds(&node, socks, http)
            .map_err(AppError::InvalidArgument)?;
        if let Some((host, port, method, password, network)) = &shadowsocks {
            xray_config.inbounds.push(Inbound {
                tag: "shadowsocks-in".to_string(),
                port: *port,
                listen: (*host).to_string(),
                protocol: "shadowsocks".to_string(),
                settings: Some(serde_json::json!({
                    "method": method,
                    "password": password,
                    "network": network
                })),
            });
        }

        let (ready_host, ready_port) = if let Some((host, port, _)) = socks {
            (connect_host_for_bind_host(host), port)
        } else if let Some((host, port)) = http {
            (connect_host_for_bind_host(host), port)
        } else if let Some((host, port, _, _, _)) = &shadowsocks {
            (connect_host_for_bind_host(host), *port)
        } else {
            unreachable!("validated at least one inbound")
        };
        let binary_path = match runtime.engine.as_str() {
            "xray" => self.context.runtime_paths.xray_path.clone(),
            "v2ray" => self.context.runtime_paths.v2ray_path.clone(),
            other => PathBuf::from(other),
        };

        Ok(ResolvedLaunch {
            binary_path,
            config: xray_config,
            ready_host,
            ready_port,
            endpoints: RuntimeEndpoints {
                socks: socks.map(|(host, port, _)| RuntimeEndpoint {
                    host: host.to_string(),
                    port,
                }),
                http: http.map(|(host, port)| RuntimeEndpoint {
                    host: host.to_string(),
                    port,
                }),
                shadowsocks: shadowsocks.map(|(host, port, _, _, _)| RuntimeEndpoint {
                    host: host.to_string(),
                    port,
                }),
            },
        })
    }
}

struct ResolvedLaunch {
    binary_path: PathBuf,
    config: crate::xray::XrayConfig,
    ready_host: String,
    ready_port: u16,
    endpoints: RuntimeEndpoints,
}

async fn active_session_state(context: &AppContext) -> crate::app::Result<ActiveSessionState> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        return Ok(ActiveSessionState::None);
    };

    if runtime_session_is_alive(&session) {
        return Ok(ActiveSessionState::Running(session));
    }

    mark_session_stale(context, &session).await?;
    Ok(ActiveSessionState::Stale(session))
}

async fn stop_active_session(context: &AppContext) -> crate::app::Result<bool> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        context.db.clear_active_config().await?;
        return Ok(false);
    };

    context
        .db
        .update_runtime_session_state(
            session.id,
            RuntimeSessionStatus::Stopping,
            None,
            None,
            None,
            None,
        )
        .await?;

    if let Some(pid) = session.process_id {
        let outcome = xray_runtime::terminate_process_gracefully(pid, SHUTDOWN_TIMEOUT)?;
        tracing::info!(
            session_id = session.id,
            pid,
            outcome = ?outcome,
            "runtime process termination completed"
        );
    } else {
        tracing::warn!(
            session_id = session.id,
            "runtime session has no saved process id"
        );
    }

    context
        .db
        .mark_runtime_session_stopped(session.id, Some(&now_string()))
        .await?;
    context.db.clear_active_config().await?;
    Ok(true)
}

async fn mark_session_stale(
    context: &AppContext,
    session: &RuntimeSessionRecord,
) -> crate::app::Result<()> {
    let terminal_status = match session.status {
        RuntimeSessionStatus::Stopping => RuntimeSessionStatus::Stopped,
        RuntimeSessionStatus::Starting | RuntimeSessionStatus::Running => {
            RuntimeSessionStatus::Failed
        }
        RuntimeSessionStatus::Stopped | RuntimeSessionStatus::Failed => return Ok(()),
    };

    context
        .db
        .update_runtime_session_state(
            session.id,
            terminal_status,
            None,
            None,
            Some(&now_string()),
            Some(stale_session_reason(session)),
        )
        .await?;
    context.db.clear_active_config().await?;
    Ok(())
}

fn stale_session_reason(session: &RuntimeSessionRecord) -> &'static str {
    match session.status {
        RuntimeSessionStatus::Stopping => "runtime process disappeared while stopping",
        RuntimeSessionStatus::Starting | RuntimeSessionStatus::Running => {
            "runtime process is not running"
        }
        RuntimeSessionStatus::Stopped | RuntimeSessionStatus::Failed => "runtime session is closed",
    }
}

fn runtime_session_is_alive(session: &RuntimeSessionRecord) -> bool {
    session
        .process_id
        .map(xray_runtime::process_is_running)
        .unwrap_or(false)
}

fn runtime_status_label(
    latest: &Option<RuntimeSessionRecord>,
    active_state: &ActiveSessionState,
    pid_running: bool,
    inbound_health: &RuntimeInboundHealth,
) -> RuntimeStatusLabel {
    match latest {
        None => RuntimeStatusLabel::Stopped,
        Some(_) if matches!(active_state, ActiveSessionState::Stale(_)) => {
            RuntimeStatusLabel::StaleReconciled
        }
        Some(session)
            if matches!(session.status, RuntimeSessionStatus::Running) && !pid_running =>
        {
            RuntimeStatusLabel::Stale
        }
        Some(session)
            if matches!(session.status, RuntimeSessionStatus::Running)
                && inbound_health.has_unreachable_endpoint() =>
        {
            RuntimeStatusLabel::Degraded
        }
        Some(session) => RuntimeStatusLabel::Persisted(session.status.clone()),
    }
}

async fn check_runtime_inbounds(
    session: &RuntimeSessionRecord,
    pid_running: bool,
) -> RuntimeInboundHealth {
    RuntimeInboundHealth {
        socks: check_runtime_inbound(
            session.socks_host.as_deref(),
            session.socks_port,
            pid_running,
        )
        .await,
        http: check_runtime_inbound(session.http_host.as_deref(), session.http_port, pid_running)
            .await,
        shadowsocks: check_runtime_inbound(
            session.shadowsocks_host.as_deref(),
            session.shadowsocks_port,
            pid_running,
        )
        .await,
    }
}

async fn check_runtime_inbound(
    host: Option<&str>,
    port: Option<i64>,
    pid_running: bool,
) -> Option<RuntimeEndpointHealth> {
    let endpoint = endpoint_from_parts(host, port)?;
    let state = if pid_running {
        let connect_host = connect_host_for_bind_host(&endpoint.host);
        match timeout(
            INBOUND_LIVENESS_TIMEOUT,
            TcpStream::connect((connect_host.as_str(), endpoint.port)),
        )
        .await
        {
            Ok(Ok(_)) => RuntimeEndpointState::Reachable,
            Ok(Err(_)) | Err(_) => RuntimeEndpointState::Unreachable,
        }
    } else {
        RuntimeEndpointState::NotChecked
    };

    Some(RuntimeEndpointHealth { endpoint, state })
}

fn endpoint_from_parts(host: Option<&str>, port: Option<i64>) -> Option<RuntimeEndpoint> {
    let port = u16::try_from(port?).ok()?;
    Some(RuntimeEndpoint {
        host: host.unwrap_or("unknown").to_string(),
        port,
    })
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn node_from_record(config: &ConfigRecord) -> crate::app::Result<Node> {
    let protocol = match config.protocol.as_str() {
        "vless" => Protocol::Vless,
        "vmess" => Protocol::Vmess,
        "ss" => Protocol::Ss,
        "trojan" => Protocol::Trojan,
        "http" => Protocol::Http,
        "socks5" => Protocol::Socks5,
        other => return Err(AppError::UnsupportedProtocol(other.to_string())),
    };

    Ok(Node {
        protocol,
        address: config.address.clone(),
        port: config.port as u16,
        username: config.username.clone(),
        uuid: config.uuid.clone(),
        password: config.password.clone(),
        method: config.method.clone(),
        network: config.network.clone(),
        tls: config.tls.clone(),
        sni: config.sni.clone(),
        host: config.host.clone(),
        path: config.path.clone(),
        name: config.name.clone(),
        raw_config: config.raw_config.clone(),
    })
}

fn connect_host_for_bind_host(host: &str) -> String {
    match host {
        "0.0.0.0" => "127.0.0.1".to_string(),
        "::" => "::1".to_string(),
        _ => host.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::AppConfig;
    use crate::app::runtime::{AppContext, RuntimePaths};
    use crate::db::{
        Database, DatabaseConnectionConfig, ImportSource, RuntimeSessionInsert, SourceKind,
    };

    #[tokio::test]
    async fn marks_running_session_with_dead_pid_as_failed() {
        let context = test_context().await;
        let summary = context
            .db
            .import_nodes(&test_source(), &[test_node()])
            .await
            .expect("node should import");
        let config = context
            .db
            .list_configs(&Default::default())
            .await
            .expect("configs should load")
            .into_iter()
            .next()
            .expect("config should exist");
        assert_eq!(summary.imported_configs, 1);
        context
            .db
            .set_active_config(config.id)
            .await
            .expect("active config should be set");
        context
            .db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(config.id),
                status: RuntimeSessionStatus::Running,
                socks_host: Some("127.0.0.1".to_string()),
                socks_port: Some(1080),
                http_host: None,
                http_port: None,
                shadowsocks_host: None,
                shadowsocks_port: None,
                process_id: Some(0),
                failure_reason: None,
                started_at: Some("1".to_string()),
                stopped_at: None,
            })
            .await
            .expect("session should insert");

        let state = RuntimeService::new(&context)
            .active_session_state()
            .await
            .expect("session state should resolve");

        assert!(matches!(state, ActiveSessionState::Stale(_)));
        assert_eq!(
            context
                .db
                .get_latest_runtime_session()
                .await
                .expect("latest should load")
                .expect("latest should exist")
                .status,
            RuntimeSessionStatus::Failed
        );
        assert!(
            context
                .db
                .get_active_config()
                .await
                .expect("active should load")
                .is_none()
        );
    }

    #[test]
    fn running_session_with_unreachable_inbound_is_degraded() {
        let session = runtime_session_with_status(RuntimeSessionStatus::Running);
        let health = RuntimeInboundHealth {
            socks: Some(RuntimeEndpointHealth {
                endpoint: RuntimeEndpoint {
                    host: "127.0.0.1".to_string(),
                    port: 1080,
                },
                state: RuntimeEndpointState::Unreachable,
            }),
            http: None,
            shadowsocks: None,
        };

        assert_eq!(
            runtime_status_label(&Some(session), &ActiveSessionState::None, true, &health),
            RuntimeStatusLabel::Degraded
        );
    }

    #[test]
    fn running_session_with_reachable_inbounds_keeps_persisted_status() {
        let session = runtime_session_with_status(RuntimeSessionStatus::Running);
        let health = RuntimeInboundHealth {
            socks: Some(RuntimeEndpointHealth {
                endpoint: RuntimeEndpoint {
                    host: "127.0.0.1".to_string(),
                    port: 1080,
                },
                state: RuntimeEndpointState::Reachable,
            }),
            http: None,
            shadowsocks: None,
        };

        assert_eq!(
            runtime_status_label(&Some(session), &ActiveSessionState::None, true, &health),
            RuntimeStatusLabel::Persisted(RuntimeSessionStatus::Running)
        );
    }

    #[test]
    fn rejects_unknown_protocol() {
        let record = ConfigRecord {
            id: 1,
            subscription_id: None,
            dedup_key: "key".to_string(),
            protocol: "unknown".to_string(),
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: None,
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
            name: None,
            raw_config: "raw".to_string(),
            is_active: false,
            is_enabled: true,
            is_selected: false,
            imported_at: "now".to_string(),
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        };

        assert!(matches!(
            node_from_record(&record),
            Err(AppError::UnsupportedProtocol(_))
        ));
    }

    #[test]
    fn maps_wildcard_bind_hosts_to_loopback_for_readiness() {
        assert_eq!(connect_host_for_bind_host("0.0.0.0"), "127.0.0.1");
        assert_eq!(connect_host_for_bind_host("::"), "::1");
        assert_eq!(connect_host_for_bind_host("127.0.0.1"), "127.0.0.1");
    }

    fn runtime_session_with_status(status: RuntimeSessionStatus) -> RuntimeSessionRecord {
        RuntimeSessionRecord {
            id: 1,
            config_id: None,
            status,
            socks_host: Some("127.0.0.1".to_string()),
            socks_port: Some(1080),
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: Some(i64::from(std::process::id())),
            failure_reason: None,
            started_at: Some("1".to_string()),
            stopped_at: None,
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        }
    }

    async fn test_context() -> AppContext {
        let root = std::env::temp_dir().join(format!(
            "xrat-runtime-service-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("root should be created");
        let database_config = DatabaseConnectionConfig::Sqlite {
            path: root.join("db.sqlite"),
        };
        let db = Database::connect(&database_config)
            .await
            .expect("database should connect");

        AppContext {
            db,
            app_config: AppConfig::default(),
            runtime_paths: RuntimePaths {
                root_dir: root.clone(),
                database_config,
                database_path: root.join("db.sqlite"),
                database_label: root.join("db.sqlite").display().to_string(),
                config_path: root.join("config.toml"),
                runtime_dir: root.join("runtime"),
                xray_path: "xray".into(),
                v2ray_path: "v2ray".into(),
            },
        }
    }

    fn test_source() -> ImportSource {
        ImportSource {
            kind: SourceKind::RawText,
            value: "test".to_string(),
            name: Some("test".to_string()),
        }
    }

    fn test_node() -> Node {
        Node {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("00000000-0000-0000-0000-000000000000".to_string()),
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("example.com".to_string()),
            host: None,
            path: None,
            name: Some("test".to_string()),
            raw_config:
                "vless://00000000-0000-0000-0000-000000000000@example.com:443?security=tls#test"
                    .to_string(),
        }
    }
}
