mod commands;
mod config_list;
mod defaults;
mod lifecycle;
mod navigation;
mod query;
mod settings;
mod tasks;
mod test_state;
mod types;
mod views;

pub use types::{
    BulkKind, BulkOp, ChromeMessage, ConfigFilter, ConfigListState, ConfigSort, ConfirmKind,
    ConfirmState, ImportModalState, ImportModalStep, PanelScroll, PanelViewport, QrKind,
    QrModalState, RenameModalState, SettingsEditState, SettingsModalState, SettingsMode,
    SettingsPane, SourceFilter, SourceListState, TestMode, TestScope, TestViewState, TuiAction,
    TuiApp, TuiConfigCommand, TuiLogTab, TuiPanel, TuiView,
};

use crate::tui::task::TuiTaskState;

impl TuiApp {
    pub fn apply(&mut self, action: TuiAction) {
        match action {
            TuiAction::Quit => self.should_quit = true,
            TuiAction::ShowHelp => self.show_help(),
            TuiAction::Back => self.back(),
            TuiAction::MoveDown => self.move_focus(1),
            TuiAction::MoveUp => self.move_focus(-1),
            TuiAction::PageDown => self.page_focus(1),
            TuiAction::PageUp => self.page_focus(-1),
            TuiAction::MoveTop => self.move_to_top(),
            TuiAction::MoveBottom => self.move_to_bottom(),
            TuiAction::BeginSearch => self.begin_search(),
            TuiAction::SearchInput(ch) => self.push_search_char(ch),
            TuiAction::SearchBackspace => self.pop_search_char(),
            TuiAction::ClearSearch => self.clear_search(),
            TuiAction::ConfirmSearch => self.close_search(),
            TuiAction::CycleSort => self.cycle_config_sort(),
            TuiAction::CycleFilter => self.cycle_config_filter(),
            TuiAction::CycleProtocolFilter => self.cycle_protocol_filter(),
            TuiAction::ToggleDeletedFilter => self.toggle_deleted_filter(),
            TuiAction::FocusNextPanel => self.focus_panel(self.focused_panel.next()),
            TuiAction::FocusPrevPanel => self.focus_panel(self.focused_panel.prev()),
            TuiAction::FocusPanel(panel) => self.focus_panel(panel),
            TuiAction::RequestDeleteFocused => self.request_delete_focused(),
            TuiAction::RequestPurgeFocused => self.request_purge_focused(),
            TuiAction::RequestDeleteSource => self.request_delete_source(),
            TuiAction::RuntimeStop
            | TuiAction::RuntimeRestart
            | TuiAction::RefreshFocusedSource
            | TuiAction::RefreshAllSources
            | TuiAction::OpenQrFocused
            | TuiAction::CopyFocused
            | TuiAction::OpenQrApiUrl
            | TuiAction::CopyApiUrl
            | TuiAction::OpenImportModal
            | TuiAction::OpenSettingsModal
            | TuiAction::OpenRenameModal
            | TuiAction::ImportSubmit
            | TuiAction::RenameSubmit
            | TuiAction::SettingsSave => {}
            TuiAction::NextLogTab => {
                self.active_log_tab = self.active_log_tab.next();
                self.panel_scroll.log.set(0);
            }
            TuiAction::PrevLogTab => {
                self.active_log_tab = self.active_log_tab.prev();
                self.panel_scroll.log.set(0);
            }
            TuiAction::RequestClearEvents => self.request_clear_events(),
            TuiAction::ClearLogView => self.clear_log_view(),
            TuiAction::ClearStatsView => self.stats.clear(),
            TuiAction::StartTest(scope) => {
                self.test_state.scope = scope;
            }
            TuiAction::RequestBulk(op) => self.request_bulk(op),
            TuiAction::ConfirmBulk => self.pending_bulk = None,
            TuiAction::CancelBulk => {
                self.pending_bulk = None;
            }
            TuiAction::ImportInput(ch) => self.append_import_text(&ch.to_string()),
            TuiAction::ImportBackspace => {
                if let Some(modal) = &mut self.import_modal {
                    modal.input.pop();
                    modal.error = None;
                }
            }
            TuiAction::RenameInput(ch) => {
                if let Some(modal) = &mut self.rename_modal {
                    modal.input.push(ch);
                    modal.error = None;
                }
            }
            TuiAction::RenameBackspace => {
                if let Some(modal) = &mut self.rename_modal {
                    modal.input.pop();
                }
            }
            TuiAction::SettingsMove(direction) => self.settings_move(direction),
            TuiAction::SettingsSwitchPane => self.settings_switch_pane(),
            TuiAction::SettingsFocusSections => self.settings_focus_pane(SettingsPane::Sections),
            TuiAction::SettingsFocusFields => self.settings_focus_pane(SettingsPane::Fields),
            TuiAction::SettingsBeginSearch => self.settings_begin_search(),
            TuiAction::SettingsInput(ch) => self.settings_input(ch),
            TuiAction::SettingsBackspace => self.settings_backspace(),
            TuiAction::SettingsClearInput => self.settings_clear_input(),
            TuiAction::SettingsSubmit => self.settings_submit(),
            TuiAction::SettingsCycle(direction) => self.settings_cycle(direction),
            TuiAction::SettingsReset => self.settings_reset(),
            TuiAction::SettingsConfirmDiscard(discard) => self.settings_confirm_discard(discard),
            TuiAction::CancelTestBatch => {
                if self.task_state.running.is_some() {
                    self.task_state.cancel();
                }
            }
            TuiAction::Confirm => self.confirm = None,
            TuiAction::Cancel => {
                self.confirm = None;
                self.needs_full_clear = true;
            }
            TuiAction::StartFocused
            | TuiAction::EnableFocused
            | TuiAction::DisableFocused
            | TuiAction::RestoreFocused => {}
            TuiAction::NextTab => self.switch_view(self.active_view.next()),
            TuiAction::PrevTab => self.switch_view(self.active_view.prev()),
            TuiAction::None => {}
        }
    }

    pub fn reload_logs(&mut self, logs: crate::tui::data::TuiLogs) {
        self.data.logs = logs;
    }

    pub fn take_needs_full_clear(&mut self) -> bool {
        let needs_clear = self.needs_full_clear;
        self.needs_full_clear = false;
        needs_clear
    }

    /// Advance the spinner and expire chrome messages. Returns `true` if visible
    /// state changed so the caller can drive dirty-flag rendering.
    pub fn tick(&mut self) -> bool {
        use crate::tui::task::TuiTaskKind;
        let mut changed = false;
        if matches!(
            self.task_state.running,
            Some(TuiTaskKind::TestBatch)
                | Some(TuiTaskKind::RuntimeOp)
                | Some(TuiTaskKind::SourceRefresh)
                | Some(TuiTaskKind::Import)
        ) {
            self.spinner_tick = self.spinner_tick.wrapping_add(1);
            changed = true;
        }
        if self
            .chrome_message
            .as_ref()
            .is_some_and(|message| std::time::Instant::now() >= message.expires_at)
        {
            self.chrome_message = None;
            changed = true;
        }
        changed
    }

    /// Current Unicode spinner frame, advanced by [`TuiApp::tick`] while a task
    /// is in flight.
    pub fn spinner_frame(&self) -> &'static str {
        const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        FRAMES[self.spinner_tick % FRAMES.len()]
    }

    /// True while a runtime switch/start/stop task is in flight.
    pub fn runtime_op_in_flight(&self) -> bool {
        self.task_state.running == Some(crate::tui::task::TuiTaskKind::RuntimeOp)
    }

    pub fn subscriptions_refresh_in_flight(&self) -> bool {
        self.task_state.running == Some(crate::tui::task::TuiTaskKind::SourceRefresh)
    }

    pub fn append_import_text(&mut self, text: &str) {
        if let Some(modal) = &mut self.import_modal {
            modal.input.push_str(text);
            modal.error = None;
        }
    }

    pub fn runtime_activity_in_flight(&self) -> bool {
        self.runtime_op_in_flight()
            || self.data.runtime.status == "starting"
            || self.data.runtime.status == "stopping"
    }

    pub fn is_testing_config(&self, config_id: i64) -> bool {
        self.task_state.running == Some(crate::tui::task::TuiTaskKind::TestBatch)
            && self.testing_config_ids.contains(&config_id)
    }
}

#[cfg(test)]
mod tests;
