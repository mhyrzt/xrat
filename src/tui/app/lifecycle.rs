use crate::tui::data::TuiData;

use super::{TuiApp, TuiView};

impl TuiApp {
    pub fn reload_data(&mut self, data: TuiData) {
        self.data = data;
        self.clamp_config_focus();
        self.clamp_source_focus();
        self.panel_scroll.detail.set(0);
        self.panel_scroll.log.set(0);
        self.panel_scroll.runtime.set(0);
    }

    pub(super) fn show_help(&mut self) {
        self.show_help = true;
        self.config_list.editing_search = false;
        self.confirm = None;
        self.needs_full_clear = true;
    }

    pub(super) fn back(&mut self) {
        if self.qr_modal.is_some() {
            self.qr_modal = None;
            self.needs_full_clear = true;
        } else if self.rename_modal.is_some() {
            self.rename_modal = None;
            self.needs_full_clear = true;
        } else if self.confirm.is_some() {
            self.confirm = None;
            self.needs_full_clear = true;
        } else if self.config_list.editing_search {
            self.close_search();
        } else {
            if self.show_help {
                self.needs_full_clear = true;
            }
            self.show_help = false;
        }
    }

    pub(super) fn begin_search(&mut self) {
        if self.active_view == TuiView::Configs {
            self.config_list.editing_search = true;
            self.show_help = false;
        }
    }

    pub(super) fn switch_view(&mut self, view: TuiView) {
        self.active_view = view;
        self.show_help = false;
        self.config_list.editing_search = false;
        self.confirm = None;
        self.needs_full_clear = true;
        self.panel_scroll.detail.set(0);
        match view {
            TuiView::Configs => self.clamp_config_focus(),
            TuiView::Sources => self.clamp_source_focus(),
        }
    }
}
