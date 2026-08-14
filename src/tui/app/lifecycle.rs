use crate::tui::data::{TuiConfigRow, TuiData};

use super::{TuiApp, TuiLogTab, TuiView};

impl TuiApp {
    /// Clear the visible buffer of the active log tab without touching persisted
    /// records. Events/api tabs advance an id watermark; the engine tab records
    /// the last visible row signature; the stats tab empties the ring buffer.
    pub(crate) fn clear_log_view(&mut self) {
        match self.active_log_tab {
            TuiLogTab::XratEvents | TuiLogTab::Api => {
                self.events_clear_before_id = self
                    .data
                    .logs
                    .events
                    .iter()
                    .map(|event| event.id)
                    .max()
                    .unwrap_or(self.events_clear_before_id);
            }
            TuiLogTab::ProxyEngine => {
                self.proxy_clear_signature = self.data.logs.proxy.last().map(|row| row.signature());
            }
            TuiLogTab::Stats => self.stats.clear(),
        }
    }

    /// True when an event row should be shown given the events view watermark.
    pub fn event_visible(&self, id: i64) -> bool {
        id > self.events_clear_before_id
    }

    pub fn reload_data(&mut self, data: TuiData) {
        self.data = data;
        self.clamp_config_focus();
        self.clamp_source_focus();
        self.panel_scroll.detail.set(0);
        self.panel_scroll.log.set(0);
        self.panel_scroll.runtime.set(0);
    }

    pub fn record_stats(
        &mut self,
        session_id: Option<i64>,
        sample: crate::xray::stats::StatsSample,
    ) {
        self.stats.record(session_id, sample);
    }

    pub fn replace_config_row(&mut self, row: TuiConfigRow) {
        let focused_id = self.focused_config().map(|config| config.id);
        self.data.replace_config_row(row);
        self.clamp_config_focus();
        if let Some(focused_id) = focused_id
            && let Some(position) = self
                .visible_config_indices()
                .iter()
                .position(|idx| self.data.configs[*idx].id == focused_id)
        {
            self.config_list.focused = position;
        }
    }

    pub(super) fn show_help(&mut self) {
        self.show_help = true;
        self.config_list.editing_search = false;
        self.confirm = None;
        self.needs_full_clear = true;
    }

    pub(super) fn back(&mut self) {
        if let Some(settings) = &mut self.settings_modal {
            if settings.editing.is_some() {
                settings.editing = None;
                settings.error = None;
            } else if settings.searching {
                settings.searching = false;
            } else if settings.discard_confirm {
                settings.discard_confirm = false;
            } else if settings.session.is_dirty() {
                settings.discard_confirm = true;
            } else {
                self.settings_modal = None;
                self.needs_full_clear = true;
            }
        } else if self.qr_modal.is_some() {
            self.qr_modal = None;
            self.needs_full_clear = true;
        } else if self.import_modal.is_some() {
            self.import_modal = None;
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
