mod commands;
mod config_list;
mod navigation;
mod tasks;
mod test_state;
mod types;
mod views;

pub use types::{
    ConfigListState, ConfigSort, ConfirmKind, ConfirmState, SourceListState, TestMode, TestScope,
    TestViewState, TuiAction, TuiApp, TuiConfigCommand, TuiView,
};

use crate::tui::data::TuiData;

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            active_view: TuiView::Configs,
            show_help: false,
            should_quit: false,
            status_message: "ready".to_string(),
            data: TuiData::default(),
            config_list: ConfigListState::default(),
            source_list: SourceListState::default(),
            test_state: TestViewState::default(),
            task_state: crate::tui::task::TuiTaskState::default(),
            confirm: None,
        }
    }
}

impl TuiApp {
    pub fn with_data(data: TuiData) -> Self {
        Self {
            status_message: format!("loaded {} configs", data.total_configs),
            data,
            ..Self::default()
        }
    }

    pub fn focused_config(&self) -> Option<&crate::tui::data::TuiConfigRow> {
        self.visible_config_indices()
            .get(self.config_list.focused)
            .and_then(|idx| self.data.configs.get(*idx))
    }

    pub fn visible_configs(&self) -> Vec<&crate::tui::data::TuiConfigRow> {
        self.visible_config_indices()
            .into_iter()
            .filter_map(|idx| self.data.configs.get(idx))
            .collect()
    }

    pub fn focused_source(&self) -> Option<&crate::tui::data::TuiSourceRow> {
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
        format!(
            "{search} - sort:{} - {deleted}",
            self.config_list.sort.label()
        )
    }

    pub fn test_scope_count(&self) -> usize {
        match self.test_state.scope {
            TestScope::Focused => usize::from(self.focused_config().is_some()),
            TestScope::Selected => self
                .data
                .configs
                .iter()
                .filter(|config| config.is_selected && config.is_enabled && !config.is_deleted)
                .count(),
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

    pub fn reload_data(&mut self, data: TuiData) {
        self.data = data;
        self.clamp_config_focus();
        self.clamp_source_focus();
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    pub fn apply(&mut self, action: TuiAction) {
        match action {
            TuiAction::Quit => self.should_quit = true,
            TuiAction::ShowHelp => self.show_help(),
            TuiAction::Back => self.back(),
            TuiAction::MoveDown => self.move_focus(1),
            TuiAction::MoveUp => self.move_focus(-1),
            TuiAction::BeginSearch => self.begin_search(),
            TuiAction::SearchInput(ch) => self.push_search_char(ch),
            TuiAction::SearchBackspace => self.pop_search_char(),
            TuiAction::ClearSearch => self.clear_search(),
            TuiAction::ConfirmSearch => self.close_search(),
            TuiAction::CycleSort => self.cycle_config_sort(),
            TuiAction::ToggleDeletedFilter => self.toggle_deleted_filter(),
            TuiAction::RequestDeleteFocused => self.request_delete_focused(),
            TuiAction::RequestPurgeFocused => self.request_purge_focused(),
            TuiAction::Confirm => self.confirm = None,
            TuiAction::Cancel => {
                self.confirm = None;
                self.status_message = "cancelled".to_string();
            }
            TuiAction::SelectFocused
            | TuiAction::EnableFocused
            | TuiAction::DisableFocused
            | TuiAction::RestoreFocused => {}
            TuiAction::SwitchView(view) => self.switch_view(view),
            TuiAction::None => {}
        }
    }

    fn show_help(&mut self) {
        self.show_help = true;
        self.config_list.editing_search = false;
        self.confirm = None;
        self.status_message = "help".to_string();
    }

    fn back(&mut self) {
        if self.confirm.is_some() {
            self.confirm = None;
            self.status_message = "cancelled".to_string();
        } else if self.config_list.editing_search {
            self.close_search();
        } else {
            self.show_help = false;
            self.status_message = "ready".to_string();
        }
    }

    fn begin_search(&mut self) {
        if self.active_view == TuiView::Configs {
            self.config_list.editing_search = true;
            self.show_help = false;
            self.status_message = "search configs".to_string();
        }
    }

    fn switch_view(&mut self, view: TuiView) {
        self.active_view = view;
        self.show_help = false;
        self.config_list.editing_search = false;
        self.confirm = None;
        self.status_message = format!("view: {}", view.label());
    }
}

#[cfg(test)]
mod tests;
