use crate::db::{ConfigListFilter, ConfigWithLatestTest, SubscriptionRecord};

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

#[derive(Debug, Default)]
pub struct TuiData {
    pub configs: Vec<TuiConfigRow>,
    pub sources: Vec<TuiSourceRow>,
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

        Ok(Self::from_configs_and_sources(configs, sources))
    }

    pub fn from_configs(configs: Vec<TuiConfigRow>) -> Self {
        Self::from_configs_and_sources(configs, Vec::new())
    }

    pub fn from_configs_and_sources(
        configs: Vec<TuiConfigRow>,
        sources: Vec<TuiSourceRow>,
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
            total_configs,
            enabled_configs,
            selected_configs,
            deleted_configs,
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
    use crate::db::SubscriptionRecord;

    use super::{TuiConfigRow, TuiData, TuiSourceRow};

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
}
