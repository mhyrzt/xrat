use crate::db::{ConfigListFilter, ConfigWithLatestTest};

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

#[derive(Debug, Default)]
pub struct TuiData {
    pub configs: Vec<TuiConfigRow>,
    pub total_configs: usize,
    pub enabled_configs: usize,
    pub selected_configs: usize,
    pub failed_configs: usize,
}

impl TuiData {
    pub async fn load(context: &crate::app::context::AppContext) -> crate::app::Result<Self> {
        let filter = ConfigListFilter {
            include_deleted: false,
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

        Ok(Self::from_configs(configs))
    }

    pub fn from_configs(configs: Vec<TuiConfigRow>) -> Self {
        let total_configs = configs.len();
        let enabled_configs = configs.iter().filter(|row| row.is_enabled).count();
        let selected_configs = configs.iter().filter(|row| row.is_selected).count();
        let failed_configs = configs
            .iter()
            .filter(|row| row.failure_reason.is_some())
            .count();

        Self {
            configs,
            total_configs,
            enabled_configs,
            selected_configs,
            failed_configs,
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
    use super::{TuiConfigRow, TuiData};

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
}
