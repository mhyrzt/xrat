use crate::tui::data::TuiData;

use super::{
    ConfigListState, PanelScroll, PanelViewport, SourceListState, TestViewState, TuiApp, TuiLogTab,
    TuiPanel, TuiTaskState, TuiView,
};

impl Default for TuiApp {
    fn default() -> Self {
        Self {
            active_view: TuiView::Configs,
            focused_panel: TuiPanel::Table,
            panel_scroll: PanelScroll::default(),
            panel_viewport: PanelViewport::default(),
            show_help: false,
            should_quit: false,
            data: TuiData::default(),
            config_list: ConfigListState::default(),
            source_list: SourceListState::default(),
            test_state: TestViewState::default(),
            task_state: TuiTaskState::default(),
            confirm: None,
            pending_chord: None,
            pending_bulk: None,
            active_log_tab: TuiLogTab::default(),
            import_modal: None,
            rename_modal: None,
            qr_modal: None,
            event_log: Vec::new(),
            chrome_message: None,
            needs_full_clear: true,
            testing_config_ids: Vec::new(),
            spinner_tick: 0,
            latest_version: None,
            engines: Vec::new(),
            stats: crate::tui::data::StatsHistory::default(),
            events_clear_before_id: 0,
            proxy_clear_signature: None,
        }
    }
}

impl TuiApp {
    pub fn with_data(data: TuiData) -> Self {
        Self {
            data,
            ..Self::default()
        }
    }
}
