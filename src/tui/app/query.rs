use crate::tui::data::{TuiConfigRow, TuiSourceRow};

use super::{TestScope, TuiApp};

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

    /// The Sources tab renders synthetic "All" (index 0) and "Orphans"
    /// (index 1) rows, so a real source begins at index 2.
    pub fn focused_source(&self) -> Option<&TuiSourceRow> {
        self.source_list
            .focused
            .checked_sub(2)
            .and_then(|idx| self.data.sources.get(idx))
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
