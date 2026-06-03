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
        self.needs_full_clear = true;
    }

    pub(super) fn back(&mut self) {
        if self.qr_modal.is_some() {
            self.qr_modal = None;
            self.status_message = "ready".to_string();
            self.needs_full_clear = true;
        } else if self.import_modal.is_some() {
            self.import_modal = None;
            self.status_message = "cancelled".to_string();
            self.needs_full_clear = true;
        } else if self.rename_modal.is_some() {
            self.rename_modal = None;
            self.status_message = "cancelled".to_string();
            self.needs_full_clear = true;
        } else if self.confirm.is_some() {
            self.confirm = None;
            self.status_message = "cancelled".to_string();
            self.needs_full_clear = true;
        } else if self.config_list.editing_search {
            self.close_search();
        } else {
            if self.show_help {
                self.needs_full_clear = true;
            }
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
        self.status_message.clear();
        self.needs_full_clear = true;
    }
}
