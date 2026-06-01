#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiView {
    Configs,
    Sources,
    Tests,
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiAction {
    Quit,
    ShowHelp,
    Back,
    MoveDown,
    MoveUp,
    BeginSearch,
    SearchInput(char),
    SearchBackspace,
    ClearSearch,
    ConfirmSearch,
    CycleSort,
    ToggleDeletedFilter,
    SelectFocused,
    EnableFocused,
    DisableFocused,
    RestoreFocused,
    RequestDeleteFocused,
    RequestPurgeFocused,
    Confirm,
    Cancel,
    SwitchView(TuiView),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiConfigCommand {
    Select(i64),
    Enable(i64),
    Disable(i64),
    Restore(i64),
    SoftDelete(i64),
    Purge(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmKind {
    SoftDeleteConfig(i64),
    PurgeConfig(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmState {
    pub kind: ConfirmKind,
    pub title: String,
    pub message: String,
}

#[derive(Debug)]
pub struct TuiApp {
    pub active_view: TuiView,
    pub show_help: bool,
    pub should_quit: bool,
    pub status_message: String,
    pub data: TuiData,
    pub config_list: ConfigListState,
    pub source_list: SourceListState,
    pub confirm: Option<ConfirmState>,
}

#[derive(Debug, Default)]
pub struct ConfigListState {
    pub focused: usize,
    pub search_query: String,
    pub editing_search: bool,
    pub sort: ConfigSort,
    pub include_deleted: bool,
}

#[derive(Debug, Default)]
pub struct SourceListState {
    pub focused: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigSort {
    #[default]
    RealDelay,
    Id,
    Name,
    Protocol,
}

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

    pub fn reload_data(&mut self, data: TuiData) {
        self.data = data;
        self.clamp_config_focus();
        self.clamp_source_focus();
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    pub fn pending_confirm_command(&self) -> Option<TuiConfigCommand> {
        self.confirm.as_ref().map(|confirm| match confirm.kind {
            ConfirmKind::SoftDeleteConfig(id) => TuiConfigCommand::SoftDelete(id),
            ConfirmKind::PurgeConfig(id) => TuiConfigCommand::Purge(id),
        })
    }

    pub fn config_command_for_action(&self, action: TuiAction) -> Option<TuiConfigCommand> {
        if self.active_view != TuiView::Configs
            || self.config_list.editing_search
            || self.confirm.is_some()
        {
            return None;
        }

        let config = self.focused_config()?;
        match action {
            TuiAction::SelectFocused if !config.is_deleted => {
                Some(TuiConfigCommand::Select(config.id))
            }
            TuiAction::EnableFocused if !config.is_deleted => {
                Some(TuiConfigCommand::Enable(config.id))
            }
            TuiAction::DisableFocused if !config.is_deleted => {
                Some(TuiConfigCommand::Disable(config.id))
            }
            TuiAction::RestoreFocused if config.is_deleted => {
                Some(TuiConfigCommand::Restore(config.id))
            }
            _ => None,
        }
    }

    pub fn apply(&mut self, action: TuiAction) {
        match action {
            TuiAction::Quit => self.should_quit = true,
            TuiAction::ShowHelp => {
                self.show_help = true;
                self.config_list.editing_search = false;
                self.confirm = None;
                self.status_message = "help".to_string();
            }
            TuiAction::Back => {
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
            TuiAction::MoveDown => self.move_focus(1),
            TuiAction::MoveUp => self.move_focus(-1),
            TuiAction::BeginSearch => {
                if self.active_view == TuiView::Configs {
                    self.config_list.editing_search = true;
                    self.show_help = false;
                    self.status_message = "search configs".to_string();
                }
            }
            TuiAction::SearchInput(ch) => self.push_search_char(ch),
            TuiAction::SearchBackspace => self.pop_search_char(),
            TuiAction::ClearSearch => self.clear_search(),
            TuiAction::ConfirmSearch => self.close_search(),
            TuiAction::CycleSort => self.cycle_config_sort(),
            TuiAction::ToggleDeletedFilter => self.toggle_deleted_filter(),
            TuiAction::RequestDeleteFocused => self.request_delete_focused(),
            TuiAction::RequestPurgeFocused => self.request_purge_focused(),
            TuiAction::Confirm => {
                self.confirm = None;
            }
            TuiAction::Cancel => {
                self.confirm = None;
                self.status_message = "cancelled".to_string();
            }
            TuiAction::SelectFocused
            | TuiAction::EnableFocused
            | TuiAction::DisableFocused
            | TuiAction::RestoreFocused => {}
            TuiAction::SwitchView(view) => {
                self.active_view = view;
                self.show_help = false;
                self.config_list.editing_search = false;
                self.confirm = None;
                self.status_message = format!("view: {}", view.label());
            }
            TuiAction::None => {}
        }
    }

    fn push_search_char(&mut self, ch: char) {
        if !self.config_list.editing_search || ch.is_control() {
            return;
        }

        self.config_list.search_query.push(ch);
        self.config_list.focused = 0;
        self.status_message = format!("{} visible configs", self.visible_config_indices().len());
    }

    fn pop_search_char(&mut self) {
        if !self.config_list.editing_search {
            return;
        }

        self.config_list.search_query.pop();
        self.config_list.focused = 0;
        self.status_message = format!("{} visible configs", self.visible_config_indices().len());
    }

    fn clear_search(&mut self) {
        self.config_list.search_query.clear();
        self.config_list.focused = 0;
        self.status_message = "search cleared".to_string();
    }

    fn close_search(&mut self) {
        self.config_list.editing_search = false;
        self.status_message = self.config_filter_summary();
    }

    fn cycle_config_sort(&mut self) {
        if self.active_view != TuiView::Configs || self.config_list.editing_search {
            return;
        }

        self.config_list.sort = self.config_list.sort.next();
        self.config_list.focused = 0;
        self.status_message = format!("sort: {}", self.config_list.sort.label());
    }

    fn toggle_deleted_filter(&mut self) {
        if self.active_view != TuiView::Configs || self.config_list.editing_search {
            return;
        }

        self.config_list.include_deleted = !self.config_list.include_deleted;
        self.config_list.focused = 0;
        self.status_message = if self.config_list.include_deleted {
            "showing deleted configs".to_string()
        } else {
            "hiding deleted configs".to_string()
        };
    }

    fn request_delete_focused(&mut self) {
        if self.active_view != TuiView::Configs
            || self.config_list.editing_search
            || self.confirm.is_some()
        {
            return;
        }

        let Some(config) = self.focused_config() else {
            return;
        };
        if config.is_deleted {
            self.status_message = "config is already deleted".to_string();
            return;
        }

        self.confirm = Some(ConfirmState {
            kind: ConfirmKind::SoftDeleteConfig(config.id),
            title: " Soft delete config ".to_string(),
            message: format!(
                "Soft delete #{} {}? The row will be hidden unless deleted configs are shown.",
                config.id,
                config.display_name()
            ),
        });
        self.status_message = "confirm soft delete".to_string();
    }

    fn request_purge_focused(&mut self) {
        if self.active_view != TuiView::Configs
            || self.config_list.editing_search
            || self.confirm.is_some()
        {
            return;
        }

        let Some(config) = self.focused_config() else {
            return;
        };
        self.confirm = Some(ConfirmState {
            kind: ConfirmKind::PurgeConfig(config.id),
            title: " Purge config ".to_string(),
            message: format!(
                "Permanently delete #{} {}? This cannot be undone.",
                config.id,
                config.display_name()
            ),
        });
        self.status_message = "confirm purge".to_string();
    }

    fn move_focus(&mut self, delta: isize) {
        match self.active_view {
            TuiView::Configs => self.move_config_focus(delta),
            TuiView::Sources => self.move_source_focus(delta),
            TuiView::Tests | TuiView::Runtime => {}
        }
    }

    fn move_config_focus(&mut self, delta: isize) {
        if self.config_list.editing_search {
            return;
        }

        let len = self.visible_config_indices().len();
        if len == 0 {
            self.config_list.focused = 0;
            return;
        }

        let next = if delta.is_negative() {
            self.config_list
                .focused
                .saturating_sub(delta.unsigned_abs())
        } else {
            (self.config_list.focused + delta as usize).min(len - 1)
        };

        self.config_list.focused = next;
        if let Some(config) = self.focused_config() {
            self.status_message = format!("#{} {}", config.id, config.display_name());
        }
    }

    fn move_source_focus(&mut self, delta: isize) {
        let len = self.data.sources.len();
        if len == 0 {
            self.source_list.focused = 0;
            return;
        }

        let next = if delta.is_negative() {
            self.source_list
                .focused
                .saturating_sub(delta.unsigned_abs())
        } else {
            (self.source_list.focused + delta as usize).min(len - 1)
        };

        self.source_list.focused = next;
        if let Some(source) = self.focused_source() {
            self.status_message = format!("source #{} {}", source.id, source.display_name());
        }
    }

    fn clamp_config_focus(&mut self) {
        let len = self.visible_config_indices().len();
        if len == 0 {
            self.config_list.focused = 0;
        } else if self.config_list.focused >= len {
            self.config_list.focused = len - 1;
        }
    }

    fn clamp_source_focus(&mut self) {
        let len = self.data.sources.len();
        if len == 0 {
            self.source_list.focused = 0;
        } else if self.source_list.focused >= len {
            self.source_list.focused = len - 1;
        }
    }

    fn visible_config_indices(&self) -> Vec<usize> {
        let query = self.config_list.search_query.trim().to_lowercase();
        let mut indices: Vec<usize> = self
            .data
            .configs
            .iter()
            .enumerate()
            .filter_map(|(idx, config)| {
                if query.is_empty() || config.matches_search(&query) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        indices.sort_by(|left, right| {
            let left = &self.data.configs[*left];
            let right = &self.data.configs[*right];
            self.config_list.sort.compare(left, right)
        });
        indices
    }
}

impl ConfigSort {
    fn next(self) -> Self {
        match self {
            Self::RealDelay => Self::Id,
            Self::Id => Self::Name,
            Self::Name => Self::Protocol,
            Self::Protocol => Self::RealDelay,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::RealDelay => "real-delay",
            Self::Id => "id",
            Self::Name => "name",
            Self::Protocol => "protocol",
        }
    }

    fn compare(
        self,
        left: &crate::tui::data::TuiConfigRow,
        right: &crate::tui::data::TuiConfigRow,
    ) -> std::cmp::Ordering {
        match self {
            Self::RealDelay => (left.real_delay_ms.unwrap_or(i64::MAX), left.id)
                .cmp(&(right.real_delay_ms.unwrap_or(i64::MAX), right.id)),
            Self::Id => left.id.cmp(&right.id),
            Self::Name => left
                .display_name()
                .to_lowercase()
                .cmp(&right.display_name().to_lowercase())
                .then_with(|| left.id.cmp(&right.id)),
            Self::Protocol => left
                .protocol
                .cmp(&right.protocol)
                .then_with(|| left.id.cmp(&right.id)),
        }
    }
}

impl TuiView {
    pub fn label(self) -> &'static str {
        match self {
            Self::Configs => "configs",
            Self::Sources => "sources",
            Self::Tests => "tests",
            Self::Runtime => "runtime",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            Self::Configs => "[CONFIGS]",
            Self::Sources => "[SOURCES]",
            Self::Tests => "[TESTS]",
            Self::Runtime => "[RUNTIME]",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tui::data::{TuiConfigRow, TuiData, TuiSourceRow};

    use super::{ConfigSort, ConfirmKind, TuiAction, TuiApp, TuiConfigCommand, TuiView};

    #[test]
    fn switches_active_view() {
        let mut app = TuiApp::default();

        app.apply(TuiAction::SwitchView(TuiView::Runtime));

        assert_eq!(app.active_view, TuiView::Runtime);
        assert_eq!(app.status_message, "view: runtime");
    }

    #[test]
    fn back_closes_help() {
        let mut app = TuiApp::default();

        app.apply(TuiAction::ShowHelp);
        app.apply(TuiAction::Back);

        assert!(!app.show_help);
        assert_eq!(app.status_message, "ready");
    }

    #[test]
    fn moves_config_focus_within_bounds() {
        let data = TuiData::from_configs(vec![row(1), row(2)]);
        let mut app = TuiApp::with_data(data);

        app.apply(TuiAction::MoveDown);
        app.apply(TuiAction::MoveDown);
        assert_eq!(app.config_list.focused, 1);

        app.apply(TuiAction::MoveUp);
        app.apply(TuiAction::MoveUp);
        assert_eq!(app.config_list.focused, 0);
    }

    #[test]
    fn moves_source_focus_within_bounds() {
        let data = TuiData::from_configs_and_sources(vec![], vec![source(1), source(2)]);
        let mut app = TuiApp::with_data(data);
        app.apply(TuiAction::SwitchView(TuiView::Sources));

        app.apply(TuiAction::MoveDown);
        app.apply(TuiAction::MoveDown);
        assert_eq!(app.source_list.focused, 1);

        app.apply(TuiAction::MoveUp);
        app.apply(TuiAction::MoveUp);
        assert_eq!(app.source_list.focused, 0);
    }

    #[test]
    fn filters_visible_configs_by_search_text() {
        let mut trojan = row(2);
        trojan.name = "fast trojan".to_string();
        trojan.protocol = "trojan".to_string();
        let data = TuiData::from_configs(vec![row(1), trojan]);
        let mut app = TuiApp::with_data(data);

        app.apply(TuiAction::BeginSearch);
        for ch in "trojan".chars() {
            app.apply(TuiAction::SearchInput(ch));
        }

        let visible: Vec<_> = app
            .visible_configs()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(visible, vec![2]);
        assert_eq!(app.focused_config().map(|row| row.id), Some(2));
    }

    #[test]
    fn clearing_search_restores_visible_configs() {
        let data = TuiData::from_configs(vec![row(1), row(2)]);
        let mut app = TuiApp::with_data(data);

        app.apply(TuiAction::BeginSearch);
        app.apply(TuiAction::SearchInput('z'));
        assert!(app.visible_configs().is_empty());

        app.apply(TuiAction::ClearSearch);

        assert_eq!(app.visible_configs().len(), 2);
    }

    #[test]
    fn cycles_config_sort_order() {
        let mut app = TuiApp::with_data(TuiData::from_configs(vec![row(2), row(1)]));

        assert_eq!(app.config_list.sort, ConfigSort::RealDelay);
        app.apply(TuiAction::CycleSort);

        assert_eq!(app.config_list.sort, ConfigSort::Id);
        let visible: Vec<_> = app
            .visible_configs()
            .into_iter()
            .map(|row| row.id)
            .collect();
        assert_eq!(visible, vec![1, 2]);
    }

    #[test]
    fn maps_focused_config_actions_to_commands() {
        let data = TuiData::from_configs(vec![row(1)]);
        let app = TuiApp::with_data(data);

        assert_eq!(
            app.config_command_for_action(TuiAction::SelectFocused),
            Some(TuiConfigCommand::Select(1))
        );
        assert_eq!(
            app.config_command_for_action(TuiAction::EnableFocused),
            Some(TuiConfigCommand::Enable(1))
        );
        assert_eq!(
            app.config_command_for_action(TuiAction::DisableFocused),
            Some(TuiConfigCommand::Disable(1))
        );
    }

    #[test]
    fn opens_and_cancels_delete_confirmation() {
        let data = TuiData::from_configs(vec![row(1)]);
        let mut app = TuiApp::with_data(data);

        app.apply(TuiAction::RequestDeleteFocused);

        assert_eq!(
            app.confirm.as_ref().map(|confirm| confirm.kind),
            Some(ConfirmKind::SoftDeleteConfig(1))
        );
        assert_eq!(
            app.pending_confirm_command(),
            Some(TuiConfigCommand::SoftDelete(1))
        );

        app.apply(TuiAction::Cancel);

        assert!(app.confirm.is_none());
        assert_eq!(app.status_message, "cancelled");
    }

    #[test]
    fn toggles_deleted_filter_and_resets_focus() {
        let data = TuiData::from_configs(vec![row(1), row(2)]);
        let mut app = TuiApp::with_data(data);
        app.apply(TuiAction::MoveDown);

        app.apply(TuiAction::ToggleDeletedFilter);

        assert!(app.config_list.include_deleted);
        assert_eq!(app.config_list.focused, 0);
    }

    #[test]
    fn restore_command_only_applies_to_deleted_configs() {
        let mut deleted = row(1);
        deleted.is_deleted = true;
        let data = TuiData::from_configs(vec![deleted]);
        let app = TuiApp::with_data(data);

        assert_eq!(
            app.config_command_for_action(TuiAction::RestoreFocused),
            Some(TuiConfigCommand::Restore(1))
        );
    }

    fn row(id: i64) -> TuiConfigRow {
        TuiConfigRow {
            id,
            name: format!("config-{id}"),
            protocol: "vless".to_string(),
            address: "example.com".to_string(),
            port: 443,
            network: "ws".to_string(),
            tls: Some("tls".to_string()),
            real_delay_ms: Some(100),
            tcp_ms: Some(20),
            failure_reason: None,
            source_id: None,
            is_active: false,
            is_enabled: true,
            is_selected: false,
            is_deleted: false,
        }
    }

    fn source(id: i64) -> TuiSourceRow {
        TuiSourceRow {
            id,
            kind: "url".to_string(),
            value: format!("https://example.com/{id}"),
            name: Some(format!("source-{id}")),
            config_count: id,
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }
}
use crate::tui::data::TuiData;
