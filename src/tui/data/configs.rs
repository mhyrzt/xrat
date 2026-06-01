use crate::db::ConfigWithLatestTest;

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
