use crate::app::runtime_service::{RuntimeEndpointHealth, RuntimeStatusSnapshot};
use crate::db::ConfigRecord;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiRuntimeStatus {
    pub status: String,
    pub pid_running: bool,
    pub session_id: Option<i64>,
    pub process_id: Option<i64>,
    pub session_status: Option<String>,
    pub active_config: Option<String>,
    pub active_config_id: Option<i64>,
    pub selected_config: Option<String>,
    pub selected_config_id: Option<i64>,
    pub session_config: Option<String>,
    pub socks: Option<String>,
    pub http: Option<String>,
    pub shadowsocks: Option<String>,
    pub started_at: Option<String>,
    pub stopped_at: Option<String>,
    pub updated_at: Option<String>,
    pub failure_reason: Option<String>,
    pub transition_reason: Option<String>,
    pub database_label: String,
}

impl Default for TuiRuntimeStatus {
    fn default() -> Self {
        Self {
            status: "unknown".to_string(),
            pid_running: false,
            session_id: None,
            process_id: None,
            session_status: None,
            active_config: None,
            active_config_id: None,
            selected_config: None,
            selected_config_id: None,
            session_config: None,
            socks: None,
            http: None,
            shadowsocks: None,
            started_at: None,
            stopped_at: None,
            updated_at: None,
            failure_reason: None,
            transition_reason: None,
            database_label: "-".to_string(),
        }
    }
}

impl From<RuntimeStatusSnapshot> for TuiRuntimeStatus {
    fn from(value: RuntimeStatusSnapshot) -> Self {
        let session = value.session;
        Self {
            status: value.status.as_str().to_string(),
            pid_running: value.pid_running,
            session_id: session.as_ref().map(|session| session.id),
            process_id: session.as_ref().and_then(|session| session.process_id),
            session_status: session
                .as_ref()
                .map(|session| session.status.as_str().to_string()),
            active_config: value.active_config.as_ref().map(config_label),
            active_config_id: value.active_config.as_ref().map(|c| c.id),
            selected_config: value.selected_config.as_ref().map(config_label),
            selected_config_id: value.selected_config.as_ref().map(|c| c.id),
            session_config: value.session_config.as_ref().map(config_label),
            socks: value
                .inbound_health
                .socks
                .as_ref()
                .map(endpoint_health_label),
            http: value
                .inbound_health
                .http
                .as_ref()
                .map(endpoint_health_label),
            shadowsocks: value
                .inbound_health
                .shadowsocks
                .as_ref()
                .map(endpoint_health_label),
            started_at: session
                .as_ref()
                .and_then(|session| session.started_at.clone()),
            stopped_at: session
                .as_ref()
                .and_then(|session| session.stopped_at.clone()),
            updated_at: session.as_ref().map(|session| session.updated_at.clone()),
            failure_reason: session
                .as_ref()
                .and_then(|session| session.failure_reason.clone()),
            transition_reason: session
                .as_ref()
                .and_then(|session| session.last_transition_reason_detail.clone())
                .or_else(|| {
                    session
                        .as_ref()
                        .and_then(|session| session.last_transition_reason_code.clone())
                }),
            database_label: value.database_label,
        }
    }
}

fn config_label(config: &ConfigRecord) -> String {
    let name = config.name.as_deref().unwrap_or("-");
    format!("#{} {name}", config.id)
}

fn endpoint_health_label(health: &RuntimeEndpointHealth) -> String {
    format!(
        "{}:{} ({})",
        health.endpoint.host,
        health.endpoint.port,
        health.state.as_str()
    )
}
