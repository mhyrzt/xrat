mod configs;
mod runtime;
mod sources;
mod tests_view;

pub use configs::TuiConfigRow;
pub use runtime::TuiRuntimeStatus;
pub use sources::TuiSourceRow;
pub use tests_view::TuiTestStatus;

use crate::app::runtime_service::RuntimeService;
use crate::db::ConfigListFilter;

#[derive(Debug, Clone)]
pub struct EngineInfo {
    pub name: &'static str,
    pub available: bool,
    pub version: Option<String>,
}

/// Minimal daemon view for the runtime card. Only carries facts not already
/// shown elsewhere (process liveness and rotation scheduling).
#[derive(Debug, Clone, Default)]
pub struct TuiDaemonInfo {
    pub running: bool,
    pub rotation_enabled: bool,
    pub interval_secs: u64,
}

#[derive(Debug, Default)]
pub struct TuiData {
    pub configs: Vec<TuiConfigRow>,
    pub sources: Vec<TuiSourceRow>,
    pub runtime: TuiRuntimeStatus,
    pub tests: TuiTestStatus,
    pub total_configs: usize,
    pub enabled_configs: usize,
    pub deleted_configs: usize,
    pub failed_configs: usize,
    pub db_label: String,
    pub config_path: String,
    pub api_b64_url: String,
    pub server_enabled: bool,
    pub daemon: TuiDaemonInfo,
}

impl TuiData {
    pub async fn load(
        context: &crate::app::context::AppContext,
        include_deleted: bool,
    ) -> crate::app::Result<Self> {
        let filter = ConfigListFilter {
            include_deleted,
            ..ConfigListFilter::default()
        };
        let mut configs: Vec<_> = context
            .db
            .list_configs_with_latest_tests(&filter)
            .await?
            .into_iter()
            .map(TuiConfigRow::from)
            .collect();

        configs.sort_by_key(|row| (row.real_delay_ms.unwrap_or(i64::MAX), row.id));
        let sources = context
            .db
            .list_subscriptions()
            .await?
            .into_iter()
            .map(TuiSourceRow::from)
            .collect();
        let runtime = RuntimeService::new(context).status().await?.into();
        let tests = TuiTestStatus::load(context, &configs).await?;

        let mut data = Self::from_parts(configs, sources, runtime, tests);
        data.db_label = context.runtime_paths.database_label.clone();
        data.config_path = context.runtime_paths.config_path.display().to_string();
        let server = &context.app_config.server;
        let api_host = match server.host.as_str() {
            "0.0.0.0" | "::" => crate::support::net::primary_local_ip()
                .unwrap_or_else(|| crate::support::net::connect_host_for_bind_host(&server.host)),
            _ => server.host.clone(),
        };
        data.api_b64_url = format!("http://{}:{}/b64", api_host, server.port);
        data.server_enabled = server.enabled;
        data.daemon = load_daemon_info(context).await;
        Ok(data)
    }

    #[allow(dead_code)]
    pub fn from_configs(configs: Vec<TuiConfigRow>) -> Self {
        Self::from_configs_and_sources(configs, Vec::new())
    }

    #[allow(dead_code)]
    pub fn from_configs_and_sources(
        configs: Vec<TuiConfigRow>,
        sources: Vec<TuiSourceRow>,
    ) -> Self {
        Self::from_parts(
            configs,
            sources,
            TuiRuntimeStatus::default(),
            TuiTestStatus::default(),
        )
    }

    pub fn from_parts(
        configs: Vec<TuiConfigRow>,
        sources: Vec<TuiSourceRow>,
        runtime: TuiRuntimeStatus,
        tests: TuiTestStatus,
    ) -> Self {
        let total_configs = configs.len();
        let enabled_configs = configs.iter().filter(|row| row.is_enabled).count();
        let deleted_configs = configs.iter().filter(|row| row.is_deleted).count();
        let failed_configs = configs
            .iter()
            .filter(|row| row.failure_reason.is_some())
            .count();

        Self {
            configs,
            sources,
            runtime,
            tests,
            total_configs,
            enabled_configs,
            deleted_configs,
            failed_configs,
            db_label: String::new(),
            config_path: String::new(),
            api_b64_url: String::new(),
            server_enabled: false,
            daemon: TuiDaemonInfo::default(),
        }
    }
}

/// Probe the proxy engines once (engines don't change during a session). Runs
/// `<bin> version` for each and records availability plus parsed version.
pub async fn probe_engines(context: &crate::app::context::AppContext) -> Vec<EngineInfo> {
    let (xray, sing_box) = tokio::join!(
        probe_engine("xray", &context.runtime_paths.xray_path),
        probe_engine("sing-box", &context.runtime_paths.sing_box_path),
    );
    vec![xray, sing_box]
}

/// Upper bound on how long a `<engine> version` probe may block startup. A hung
/// or missing binary must not freeze the TUI before its event loop starts.
const ENGINE_PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(2);

async fn probe_engine(name: &'static str, path: &std::path::Path) -> EngineInfo {
    let unavailable = EngineInfo {
        name,
        available: false,
        version: None,
    };
    let command = tokio::process::Command::new(path)
        .arg("version")
        .kill_on_drop(true)
        .output();
    match tokio::time::timeout(ENGINE_PROBE_TIMEOUT, command).await {
        Ok(Ok(output)) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout);
            EngineInfo {
                name,
                available: true,
                version: parse_engine_version(&text),
            }
        }
        _ => unavailable,
    }
}

/// Pull the first `MAJOR.MINOR...` token out of an engine's version banner.
fn parse_engine_version(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|token| token.trim_start_matches('v'))
        .find(|token| {
            token.contains('.') && token.chars().next().is_some_and(|c| c.is_ascii_digit())
        })
        .map(str::to_string)
}

/// Upper bound on the daemon status IPC during a TUI data load. A daemon that
/// accepts the connection but never replies must not freeze startup.
const DAEMON_STATUS_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(1);

async fn load_daemon_info(context: &crate::app::context::AppContext) -> TuiDaemonInfo {
    let socket = crate::app::daemon::ipc::default_socket_path(&context.runtime_paths.runtime_dir);
    let status = tokio::time::timeout(
        DAEMON_STATUS_TIMEOUT,
        crate::app::daemon::ipc::proxy_status_daemon(&socket),
    )
    .await;
    match status {
        Ok(Ok(response)) => {
            let payload = response.payload;
            TuiDaemonInfo {
                running: payload.as_ref().map(|p| p.daemon_ready).unwrap_or(false),
                rotation_enabled: payload
                    .as_ref()
                    .map(|p| p.rotation_enabled)
                    .unwrap_or(false),
                interval_secs: payload.as_ref().map(|p| p.interval_secs).unwrap_or(0),
            }
        }
        // IPC error or timeout: treat the daemon as unavailable.
        _ => TuiDaemonInfo::default(),
    }
}

#[cfg(test)]
mod tests;
