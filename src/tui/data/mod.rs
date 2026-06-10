mod configs;
mod logs;
mod proxy_log;
mod runtime;
mod sources;
mod tests_view;

pub use configs::TuiConfigRow;
pub use logs::TuiLogs;
pub use proxy_log::ProxyStream;
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
    pub logs: TuiLogs,
    pub metric_columns: TuiMetricColumns,
    pub test_stage_label: String,
    pub test_stage_names: Vec<String>,
    metric_columns_from_settings: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TuiMetricColumns {
    pub icmp: bool,
    pub tcp: bool,
    pub real_delay: bool,
    pub download: bool,
    pub upload: bool,
    pub country: bool,
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

        let logs = TuiLogs::load(context).await?;

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
        data.logs = logs;
        data.test_stage_names =
            normalize_test_stage_names(&context.app_config.runtime.rotation.test_stages);
        data.test_stage_label = format_test_stage_label(&data.test_stage_names);
        data.metric_columns =
            TuiMetricColumns::from_test_stages(&data.configs, &data.test_stage_names);
        data.metric_columns_from_settings = true;
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
        let metric_columns = TuiMetricColumns::from_configs(&configs, None);

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
            logs: TuiLogs::default(),
            metric_columns,
            test_stage_label: "tcp + real-delay".to_string(),
            test_stage_names: vec!["tcp".to_string(), "real_delay".to_string()],
            metric_columns_from_settings: false,
        }
    }

    pub fn replace_config_row(&mut self, row: TuiConfigRow) {
        if let Some(existing) = self.configs.iter_mut().find(|config| config.id == row.id) {
            *existing = row;
        } else {
            self.configs.push(row);
        }
        self.refresh_config_counts();
    }

    pub fn apply_location_meta_for(
        &mut self,
        id: i64,
        meta: crate::support::geoip::EndpointGeoMeta,
    ) {
        if let Some(config) = self.configs.iter_mut().find(|config| config.id == id) {
            config.apply_location_meta(meta);
        }
    }

    pub fn clear_test_fields_for_configs(&mut self, config_ids: &[i64]) {
        for config in self
            .configs
            .iter_mut()
            .filter(|config| config_ids.contains(&config.id))
        {
            config.clear_test_fields();
        }
        self.refresh_config_counts();
    }

    fn refresh_config_counts(&mut self) {
        self.total_configs = self.configs.len();
        self.enabled_configs = self.configs.iter().filter(|row| row.is_enabled).count();
        self.deleted_configs = self.configs.iter().filter(|row| row.is_deleted).count();
        self.failed_configs = self
            .configs
            .iter()
            .filter(|row| row.failure_reason.is_some())
            .count();
        self.tests.untested_configs = self
            .configs
            .iter()
            .filter(|config| config.real_delay_ms.is_none() && config.tcp_ms.is_none())
            .count();
        self.tests.stale_configs = self
            .configs
            .iter()
            .filter(|config| config.failure_reason.is_some())
            .count();
        if !self.metric_columns_from_settings {
            self.metric_columns = TuiMetricColumns::from_configs(&self.configs, None);
        }
    }
}

impl TuiMetricColumns {
    pub fn from_test_stages(configs: &[TuiConfigRow], stages: &[String]) -> Self {
        Self {
            icmp: stages.iter().any(|stage| stage == "icmp"),
            tcp: false,
            real_delay: stages.iter().any(|stage| stage == "real_delay"),
            download: stages.iter().any(|stage| stage == "download"),
            upload: false,
            country: has_location_data(configs),
        }
    }

    pub fn from_configs(
        configs: &[TuiConfigRow],
        settings: Option<&crate::app::config::TestingSettings>,
    ) -> Self {
        if let Some(settings) = settings {
            return Self {
                icmp: settings.icmp.enabled,
                tcp: settings.tcp.enabled,
                real_delay: settings.real_delay.enabled,
                download: settings.download.enabled,
                upload: false,
                country: has_location_data(configs),
            };
        }

        Self {
            icmp: configs.iter().any(|row| row.icmp_ms.is_some()),
            tcp: configs.iter().any(|row| row.tcp_ms.is_some()),
            real_delay: configs.iter().any(|row| row.real_delay_ms.is_some()),
            download: configs.iter().any(|row| row.download_mbps.is_some()),
            upload: configs.iter().any(|row| row.upload_mbps.is_some()),
            country: has_location_data(configs),
        }
    }
}

fn has_location_data(configs: &[TuiConfigRow]) -> bool {
    configs.iter().any(|row| {
        row.endpoint_country.is_some()
            || row.endpoint_location.is_some()
            || row.endpoint_asn.is_some()
    })
}

/// Time-to-live for in-session GeoIP lookups so repeated resolutions of the same
/// IP reuse the decoded mmdb result instead of re-reading the database.
const GEO_LOOKUP_TTL: std::time::Duration = std::time::Duration::from_secs(3600);

/// Upper bound on cached IP lookups kept in memory for a single session.
const GEO_LOOKUP_MAX_ENTRIES: usize = 8192;

/// Build the GeoIP lookup used for background location enrichment, wrapping the
/// local mmdb backend in an in-session [`CachedLookup`] so the lookup is built
/// once per session rather than rebuilt on every data load.
pub fn build_geo_lookup(
    app_config: &crate::app::config::AppConfig,
    runtime_paths: &crate::app::context::RuntimePaths,
) -> std::sync::Arc<crate::support::geoip::CachedLookup> {
    let inner = std::sync::Arc::new(local_mmdb_lookup(app_config, runtime_paths));
    std::sync::Arc::new(crate::support::geoip::CachedLookup::new(
        inner,
        GEO_LOOKUP_TTL,
        GEO_LOOKUP_MAX_ENTRIES,
    ))
}

fn local_mmdb_lookup(
    app_config: &crate::app::config::AppConfig,
    runtime_paths: &crate::app::context::RuntimePaths,
) -> crate::support::geoip::LocalMmdbLookup {
    crate::support::geoip::LocalMmdbLookup::new(
        crate::app::paths::mmdb::mmdb_path_for(
            runtime_paths,
            app_config,
            &app_config.testing.geoip.country_path,
            "GeoLite2-Country.mmdb",
        ),
        crate::app::paths::mmdb::mmdb_path_for(
            runtime_paths,
            app_config,
            &app_config.testing.geoip.city_path,
            "GeoLite2-City.mmdb",
        ),
        crate::app::paths::mmdb::mmdb_path_for(
            runtime_paths,
            app_config,
            &app_config.testing.geoip.asn_path,
            "GeoLite2-ASN.mmdb",
        ),
    )
}

fn normalize_test_stage_names(stages: &[String]) -> Vec<String> {
    let mut names = Vec::new();
    for stage in stages {
        let Some(stage) = crate::app::config::ConnectionTestStage::from_config_str(stage) else {
            continue;
        };
        let name = stage.config_name().to_string();
        if !names.contains(&name) {
            names.push(name);
        }
    }
    names
}

fn format_test_stage_label(stages: &[String]) -> String {
    if stages.is_empty() {
        return "none".to_string();
    }

    stages
        .iter()
        .map(|stage| match stage.as_str() {
            "real_delay" => "real-delay",
            other => other,
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

#[cfg(test)]
mod stage_tests {
    use super::*;

    #[test]
    fn normalizes_and_formats_tui_test_stages() {
        let stages = normalize_test_stage_names(&[
            "icmp".to_string(),
            "real-delay".to_string(),
            "icmp".to_string(),
        ]);

        assert_eq!(stages, vec!["icmp", "real_delay"]);
        assert_eq!(format_test_stage_label(&stages), "icmp + real-delay");
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
