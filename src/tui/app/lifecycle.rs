use crate::tui::data::TuiData;

use super::{TuiApp, TuiView};

impl TuiApp {
    pub fn reload_data(&mut self, data: TuiData) {
        self.data = data;
        self.clamp_config_focus();
        self.clamp_source_focus();
    }

    pub(super) fn show_help(&mut self) {
        self.show_help = true;
        self.config_list.editing_search = false;
        self.confirm = None;
        self.status_message = "help".to_string();
    }

    pub(super) fn back(&mut self) {
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

    pub(super) fn begin_search(&mut self) {
        if self.active_view == TuiView::Configs {
            self.config_list.editing_search = true;
            self.show_help = false;
            self.status_message = "search configs".to_string();
        }
    }

    pub(super) fn switch_view(&mut self, view: TuiView) {
        self.active_view = view;
        self.show_help = false;
        self.config_list.editing_search = false;
        self.confirm = None;
        self.status_message = format!("view: {}", view.label());
    }
}
