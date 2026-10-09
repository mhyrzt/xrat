use super::*;

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
    /// Probe/test history for the active config, empty when nothing is active.
    pub probe_history: TuiProbeHistory,
    pub metric_columns: TuiMetricColumns,
    pub test_stage_label: String,
    pub test_stage_names: Vec<String>,
    /// `(config_id, address)` rows still needing a network location lookup after
    /// DB test geo and the persistent cache have been applied.
    pub pending_enrichment: Vec<(xrat_model::ConfigId, String)>,
    pub(super) metric_columns_from_settings: bool,
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
        Ok(DashboardService::new(context)
            .load(include_deleted)
            .await?
            .into())
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
            probe_history: TuiProbeHistory::default(),
            metric_columns,
            test_stage_label: "tcp + real-delay".to_string(),
            test_stage_names: vec!["tcp".to_string(), "real_delay".to_string()],
            pending_enrichment: Vec::new(),
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

    pub fn set_config_enabled(&mut self, id: xrat_model::ConfigId, enabled: bool) {
        if let Some(config) = self.configs.iter_mut().find(|config| config.id == id) {
            config.is_enabled = enabled;
        }
        self.refresh_config_counts();
    }

    pub fn apply_location_meta_for(
        &mut self,
        id: xrat_model::ConfigId,
        meta: xrat_support::geoip::EndpointGeoMeta,
    ) {
        if let Some(config) = self.configs.iter_mut().find(|config| config.id == id) {
            config.apply_location_meta(meta);
        }
    }

    pub fn clear_test_fields_for_configs(&mut self, config_ids: &[xrat_model::ConfigId]) {
        for config in self
            .configs
            .iter_mut()
            .filter(|config| config_ids.contains(&config.id))
        {
            config.clear_test_fields();
        }
        self.refresh_config_counts();
    }

    pub(super) fn refresh_config_counts(&mut self) {
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

pub(super) fn has_location_data(configs: &[TuiConfigRow]) -> bool {
    configs.iter().any(|row| {
        row.dial_endpoint_country.is_some()
            || row.dial_endpoint_location.is_some()
            || row.dial_endpoint_asn.is_some()
    })
}

pub(super) fn normalize_test_stage_names(stages: &[String]) -> Vec<String> {
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

pub(super) fn format_test_stage_label(stages: &[String]) -> String {
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

impl From<DashboardSnapshot> for TuiData {
    fn from(value: DashboardSnapshot) -> Self {
        let configs = value
            .configs
            .into_iter()
            .map(TuiConfigRow::from)
            .collect::<Vec<_>>();
        let tests =
            TuiTestStatus::from_run_and_results(value.latest_run, value.test_results, &configs);
        let mut runtime =
            TuiRuntimeStatus::from_snapshot(value.runtime, value.local_address.as_deref());
        runtime.tun = value.tun_label;
        let mut data = Self::from_parts(
            configs,
            value.sources.into_iter().map(TuiSourceRow::from).collect(),
            runtime,
            tests,
        );
        data.db_label = value.db_label;
        data.config_path = value.config_path;
        data.api_b64_url = value.api_b64_url;
        data.server_enabled = value.server_enabled;
        data.daemon = value.daemon;
        data.logs = value.logs;
        data.probe_history = TuiProbeHistory::from_records(&value.probe_history);
        data.test_stage_names = normalize_test_stage_names(&value.test_stages);
        data.test_stage_label = format_test_stage_label(&data.test_stage_names);
        data.metric_columns =
            TuiMetricColumns::from_test_stages(&data.configs, &data.test_stage_names);
        data.metric_columns_from_settings = true;
        data.pending_enrichment = value.pending_enrichment;
        data
    }
}
