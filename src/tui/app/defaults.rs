use crate::tui::data::TuiData;

use super::{ConfigListState, SourceListState, TestViewState, TuiApp, TuiTaskState, TuiView};

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
            task_state: TuiTaskState::default(),
            confirm: None,
            import_modal: None,
            rename_modal: None,
            qr_modal: None,
            event_log: Vec::new(),
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
}
