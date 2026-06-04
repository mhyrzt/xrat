use crate::tui::data::{TuiConfigRow, TuiSourceRow};

use super::{ConfigFilter, TestScope, TuiApp};

impl TuiApp {
    pub fn focused_config(&self) -> Option<&TuiConfigRow> {
        self.visible_config_indices()
            .get(self.config_list.focused)
            .and_then(|idx| self.data.configs.get(*idx))
    }

    pub fn visible_configs(&self) -> Vec<&TuiConfigRow> {
        self.visible_config_indices()
            .into_iter()
            .filter_map(|idx| self.data.configs.get(idx))
            .collect()
    }

    pub fn focused_source(&self) -> Option<&TuiSourceRow> {
        self.data.sources.get(self.source_list.focused)
    }

    pub fn config_filter_summary(&self) -> String {
        let search = if self.config_list.search_query.is_empty() {
            "search:-".to_string()
        } else {
            format!("search:{}", self.config_list.search_query)
        };
        let deleted = if self.config_list.include_deleted {
            "deleted:on"
        } else {
            "deleted:off"
        };
        let filter_part = match self.config_list.filter {
            ConfigFilter::None => String::new(),
            f => format!(" - filter:{}", f.label()),
        };
        let proto_part = match &self.config_list.protocol_filter {
            None => String::new(),
            Some(p) => format!(" - proto:{p}"),
        };
        format!(
            "{search} - sort:{}{filter_part}{proto_part} - {deleted}",
            self.config_list.sort.label()
        )
    }

    pub fn test_scope_count(&self) -> usize {
        match self.test_state.scope {
            TestScope::Focused => usize::from(self.focused_config().is_some()),
            TestScope::Filtered => self
                .visible_configs()
                .into_iter()
                .filter(|config| config.is_enabled && !config.is_deleted)
                .count(),
            TestScope::AllEnabled => self
                .data
                .configs
                .iter()
                .filter(|config| config.is_enabled && !config.is_deleted)
                .count(),
            TestScope::Failed => self
                .data
                .configs
                .iter()
                .filter(|config| {
                    config.failure_reason.is_some() && config.is_enabled && !config.is_deleted
                })
                .count(),
            TestScope::Stale => self
                .data
                .configs
                .iter()
                .filter(|config| {
                    config.real_delay_ms.is_none() && config.tcp_ms.is_none() && !config.is_deleted
                })
                .count(),
        }
    }
}
