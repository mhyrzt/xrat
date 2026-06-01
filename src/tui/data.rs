use crate::app::runtime_service::{RuntimeEndpointHealth, RuntimeService, RuntimeStatusSnapshot};
use crate::db::{
    ConfigListFilter, ConfigRecord, ConfigWithLatestTest, ConnectionTestRecord,
    ConnectionTestRunRecord, SubscriptionRecord,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TuiConfigRow {
    pub id: i64,
    pub name: String,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub network: String,
    pub tls: Option<String>,
    pub real_delay_ms: Option<i64>,
    pub tcp_ms: Option<i64>,
    pub failure_reason: Option<String>,
    pub source_id: Option<i64>,
    pub is_active: bool,
    pub is_enabled: bool,
    pub is_selected: bool,
    pub is_deleted: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuiSourceRow {
    pub id: i64,
    pub kind: String,
    pub value: String,
    pub name: Option<String>,
    pub config_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiRuntimeStatus {
    pub status: String,
    pub pid_running: bool,
    pub session_id: Option<i64>,
    pub process_id: Option<i64>,
    pub session_status: Option<String>,
    pub active_config: Option<String>,
    pub selected_config: Option<String>,
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

#[derive(Debug, Clone, PartialEq)]
pub struct TuiTestStatus {
    pub latest_run_id: Option<i64>,
    pub latest_run_kind: Option<String>,
    pub latest_run_created_at: Option<String>,
    pub total_results: usize,
    pub success_results: usize,
    pub failed_results: usize,
    pub untested_configs: usize,
    pub stale_configs: usize,
    pub recent_results: Vec<TuiTestResultRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TuiTestResultRow {
    pub id: i64,
    pub config_id: i64,
    pub status: String,
    pub real_delay_ms: Option<i64>,
    pub tcp_ms: Option<i64>,
    pub failure_reason: Option<String>,
    pub tested_at: String,
}

#[derive(Debug, Default)]
pub struct TuiData {
    pub configs: Vec<TuiConfigRow>,
    pub sources: Vec<TuiSourceRow>,
    pub runtime: TuiRuntimeStatus,
    pub tests: TuiTestStatus,
    pub total_configs: usize,
    pub enabled_configs: usize,
    pub selected_configs: usize,
    pub deleted_configs: usize,
    pub failed_configs: usize,
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

        Ok(Self::from_parts(configs, sources, runtime, tests))
    }

    pub fn from_configs(configs: Vec<TuiConfigRow>) -> Self {
        Self::from_configs_and_sources(configs, Vec::new())
    }

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
        let selected_configs = configs.iter().filter(|row| row.is_selected).count();
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
            selected_configs,
            deleted_configs,
            failed_configs,
        }
    }
}

impl Default for TuiTestStatus {
    fn default() -> Self {
        Self {
            latest_run_id: None,
            latest_run_kind: None,
            latest_run_created_at: None,
            total_results: 0,
            success_results: 0,
            failed_results: 0,
            untested_configs: 0,
            stale_configs: 0,
            recent_results: Vec::new(),
        }
    }
}

impl TuiTestStatus {
    async fn load(
        context: &crate::app::context::AppContext,
        configs: &[TuiConfigRow],
    ) -> crate::app::Result<Self> {
        let latest_run = context.db.get_latest_connection_test_run().await?;
        let results = match latest_run.as_ref() {
            Some(run) => context.db.list_connection_tests_by_run(run.id).await?,
            None => Vec::new(),
        };

        Ok(Self::from_run_and_results(latest_run, results, configs))
    }

    pub fn from_run_and_results(
        run: Option<ConnectionTestRunRecord>,
        results: Vec<ConnectionTestRecord>,
        configs: &[TuiConfigRow],
    ) -> Self {
        let total_results = results.len();
        let failed_results = results
            .iter()
            .filter(|result| result.failure_reason.is_some() || result.failure_kind.is_some())
            .count();
        let success_results = total_results.saturating_sub(failed_results);
        let untested_configs = configs
            .iter()
            .filter(|config| config.real_delay_ms.is_none() && config.tcp_ms.is_none())
            .count();
        let stale_configs = configs
            .iter()
            .filter(|config| config.failure_reason.is_some())
            .count();
        let recent_results = results
            .into_iter()
            .take(8)
            .map(TuiTestResultRow::from)
            .collect();

        Self {
            latest_run_id: run.as_ref().map(|run| run.id),
            latest_run_kind: run.as_ref().map(|run| run.kind.clone()),
            latest_run_created_at: run.map(|run| run.created_at),
            total_results,
            success_results,
            failed_results,
            untested_configs,
            stale_configs,
            recent_results,
        }
    }

    pub fn progress_label(&self) -> String {
        match self.latest_run_id {
            Some(_) => format!(
                "{} done - {} ok - {} failed",
                self.total_results, self.success_results, self.failed_results
            ),
            None => "no test run yet".to_string(),
        }
    }
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
            selected_config: None,
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

impl TuiConfigRow {
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            "-"
        } else {
            &self.name
        }
    }

    pub fn endpoint(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    pub fn network_label(&self) -> String {
        match &self.tls {
            Some(tls) if !tls.is_empty() => format!("{}+{}", self.network, tls),
            _ => self.network.clone(),
        }
    }

    pub fn delay_label(&self) -> String {
        if self.failure_reason.is_some() {
            "FAIL".to_string()
        } else {
            self.real_delay_ms
                .map(|delay| format!("{delay}ms"))
                .unwrap_or_else(|| "-".to_string())
        }
    }

    pub fn status_label(&self) -> String {
        let mut flags = Vec::new();
        if self.is_deleted {
            flags.push("del");
        }
        if !self.is_enabled {
            flags.push("off");
        }
        if self.is_selected {
            flags.push("sel");
        }
        if self.is_active {
            flags.push("run");
        }
        if self.failure_reason.is_some() {
            flags.push("fail");
        }
        if flags.is_empty() {
            "ok".to_string()
        } else {
            flags.join(",")
        }
    }

    pub fn matches_search(&self, query: &str) -> bool {
        self.display_name().to_lowercase().contains(query)
            || self.protocol.to_lowercase().contains(query)
            || self.address.to_lowercase().contains(query)
            || self.network.to_lowercase().contains(query)
            || self
                .source_id
                .map(|source_id| source_id.to_string().contains(query))
                .unwrap_or(false)
    }
}

impl TuiSourceRow {
    pub fn display_name(&self) -> &str {
        self.name
            .as_deref()
            .filter(|name| !name.is_empty())
            .unwrap_or("-")
    }

    pub fn value_label(&self) -> &str {
        if self.value.is_empty() {
            "-"
        } else {
            &self.value
        }
    }
}

impl From<SubscriptionRecord> for TuiSourceRow {
    fn from(value: SubscriptionRecord) -> Self {
        Self {
            id: value.id,
            kind: value.source_kind,
            value: value.source_url.unwrap_or_default(),
            name: value.name,
            config_count: value.config_count,
            created_at: value.created_at,
            updated_at: value.updated_at,
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
            selected_config: value.selected_config.as_ref().map(config_label),
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

impl From<ConnectionTestRecord> for TuiTestResultRow {
    fn from(value: ConnectionTestRecord) -> Self {
        let status = if value.failure_reason.is_some() || value.failure_kind.is_some() {
            "fail".to_string()
        } else {
            "ok".to_string()
        };

        Self {
            id: value.id,
            config_id: value.config_id,
            status,
            real_delay_ms: value.real_delay_ms,
            tcp_ms: value.tcp_ms,
            failure_reason: value.failure_reason.or(value.failure_kind),
            tested_at: value.tested_at,
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

impl From<ConfigWithLatestTest> for TuiConfigRow {
    fn from(value: ConfigWithLatestTest) -> Self {
        let config = value.config;
        Self {
            id: config.id,
            name: config.name.unwrap_or_default(),
            protocol: config.protocol,
            address: config.address,
            port: config.port,
            network: config.network,
            tls: config.tls,
            real_delay_ms: value.real_delay_ms,
            tcp_ms: value.tcp_ms,
            failure_reason: value.failure_reason,
            source_id: config.subscription_id,
            is_active: config.is_active,
            is_enabled: config.is_enabled,
            is_selected: config.is_selected,
            is_deleted: config.is_deleted,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{ConnectionTestRecord, ConnectionTestRunRecord, SubscriptionRecord};

    use super::{TuiConfigRow, TuiData, TuiRuntimeStatus, TuiSourceRow, TuiTestStatus};

    fn row(id: i64, delay: Option<i64>) -> TuiConfigRow {
        TuiConfigRow {
            id,
            name: format!("config-{id}"),
            protocol: "vless".to_string(),
            address: "example.com".to_string(),
            port: 443,
            network: "ws".to_string(),
            tls: Some("tls".to_string()),
            real_delay_ms: delay,
            tcp_ms: Some(20),
            failure_reason: None,
            source_id: None,
            is_active: false,
            is_enabled: true,
            is_selected: id % 2 == 0,
            is_deleted: false,
        }
    }

    #[test]
    fn summarizes_config_counts() {
        let mut failed = row(3, None);
        failed.failure_reason = Some("timeout".to_string());
        failed.is_enabled = false;

        let data = TuiData::from_configs(vec![row(1, Some(100)), row(2, Some(200)), failed]);

        assert_eq!(data.total_configs, 3);
        assert_eq!(data.enabled_configs, 2);
        assert_eq!(data.selected_configs, 1);
        assert_eq!(data.deleted_configs, 0);
        assert_eq!(data.failed_configs, 1);
    }

    #[test]
    fn formats_network_delay_and_status_labels() {
        let mut active = row(4, Some(88));
        active.is_active = true;

        assert_eq!(active.network_label(), "ws+tls");
        assert_eq!(active.delay_label(), "88ms");
        assert_eq!(active.status_label(), "sel,run");
    }

    #[test]
    fn matches_searchable_config_fields() {
        let row = row(4, Some(88));

        assert!(row.matches_search("config-4"));
        assert!(row.matches_search("vless"));
        assert!(row.matches_search("example"));
        assert!(!row.matches_search("missing"));
    }

    #[test]
    fn maps_subscription_record_to_source_row() {
        let row = TuiSourceRow::from(SubscriptionRecord {
            id: 7,
            source_kind: "url".to_string(),
            source_url: Some("https://example.com/sub".to_string()),
            name: Some("main".to_string()),
            created_at: "created".to_string(),
            updated_at: "updated".to_string(),
            config_count: 42,
        });

        assert_eq!(row.id, 7);
        assert_eq!(row.display_name(), "main");
        assert_eq!(row.value_label(), "https://example.com/sub");
        assert_eq!(row.config_count, 42);
    }

    #[test]
    fn default_runtime_status_is_renderable() {
        let runtime = TuiRuntimeStatus::default();

        assert_eq!(runtime.status, "unknown");
        assert_eq!(runtime.database_label, "-");
        assert!(!runtime.pid_running);
    }

    #[test]
    fn summarizes_latest_test_run() {
        let mut untested = row(2, None);
        untested.tcp_ms = None;
        let configs = vec![row(1, Some(100)), untested];
        let failed = test_record(11, 1, Some("timeout"));
        let ok = test_record(12, 2, None);

        let status = TuiTestStatus::from_run_and_results(
            Some(ConnectionTestRunRecord {
                id: 5,
                kind: "real-delay".to_string(),
                created_at: "created".to_string(),
            }),
            vec![failed, ok],
            &configs,
        );

        assert_eq!(status.latest_run_id, Some(5));
        assert_eq!(status.total_results, 2);
        assert_eq!(status.success_results, 1);
        assert_eq!(status.failed_results, 1);
        assert_eq!(status.untested_configs, 1);
        assert_eq!(status.progress_label(), "2 done - 1 ok - 1 failed");
    }

    fn test_record(id: i64, config_id: i64, failure_reason: Option<&str>) -> ConnectionTestRecord {
        ConnectionTestRecord {
            id,
            run_id: Some(5),
            config_id,
            icmp_ok: None,
            icmp_ms: None,
            tcp_ok: Some(failure_reason.is_none()),
            tcp_ms: Some(20),
            real_delay_ok: Some(failure_reason.is_none()),
            real_delay_ms: Some(100),
            download_mbps: None,
            upload_mbps: None,
            connect_ms: None,
            ttfb_ms: None,
            http_status: None,
            endpoint_ip: None,
            endpoint_location: None,
            endpoint_country: None,
            endpoint_asn: None,
            failure_kind: None,
            failure_reason: failure_reason.map(str::to_string),
            tested_at: "tested".to_string(),
        }
    }
}
