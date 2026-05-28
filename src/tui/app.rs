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
    Search,
    SwitchView(TuiView),
    None,
}

#[derive(Debug)]
pub struct TuiApp {
    pub active_view: TuiView,
    pub show_help: bool,
    pub should_quit: bool,
    pub status_message: String,
    pub data: TuiData,
    pub config_list: ConfigListState,
}

#[derive(Debug, Default)]
pub struct ConfigListState {
    pub focused: usize,
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
        self.data.configs.get(self.config_list.focused)
    }

    pub fn apply(&mut self, action: TuiAction) {
        match action {
            TuiAction::Quit => self.should_quit = true,
            TuiAction::ShowHelp => {
                self.show_help = true;
                self.status_message = "help".to_string();
            }
            TuiAction::Back => {
                self.show_help = false;
                self.status_message = "ready".to_string();
            }
            TuiAction::MoveDown => self.move_config_focus(1),
            TuiAction::MoveUp => self.move_config_focus(-1),
            TuiAction::Search => self.status_message = "search not implemented yet".to_string(),
            TuiAction::SwitchView(view) => {
                self.active_view = view;
                self.show_help = false;
                self.status_message = format!("view: {}", view.label());
            }
            TuiAction::None => {}
        }
    }

    fn move_config_focus(&mut self, delta: isize) {
        if self.active_view != TuiView::Configs || self.data.configs.is_empty() {
            return;
        }

        let len = self.data.configs.len();
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
    use crate::tui::data::{TuiConfigRow, TuiData};

    use super::{TuiAction, TuiApp, TuiView};

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
}
use crate::tui::data::TuiData;
