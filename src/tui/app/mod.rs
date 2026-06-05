mod commands;
mod config_list;
mod defaults;
mod lifecycle;
mod navigation;
mod query;
mod tasks;
mod test_state;
mod types;
mod views;

pub use types::{
    BulkKind, BulkOp, ConfigFilter, ConfigListState, ConfigSort, ConfirmKind, ConfirmState,
    PanelScroll, QrModalState, RenameModalState, SourceFilter, SourceListState, TestMode,
    TestScope, TestViewState, TuiAction, TuiApp, TuiConfigCommand, TuiPanel, TuiView,
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
            | TuiAction::OpenRenameModal
            | TuiAction::RenameSubmit => {}
            TuiAction::StartTest(scope) => {
                self.test_state.scope = scope;
            }
            TuiAction::RequestBulk(op) => self.request_bulk(op),
            TuiAction::ConfirmBulk => self.pending_bulk = None,
            TuiAction::CancelBulk => {
                self.pending_bulk = None;
                self.status_message = "cancelled".to_string();
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
            TuiAction::CancelTestBatch => {
                if self.task_state.running.is_some() {
                    if self.task_state.cancel() {
                        self.status_message = "cancelling test batch".to_string();
                    } else {
                        self.status_message = "no cancellable task is running".to_string();
                    }
                } else {
                    self.status_message = "no test batch is running".to_string();
                }
            }
            TuiAction::Confirm => self.confirm = None,
            TuiAction::Cancel => {
                self.confirm = None;
                self.status_message = "cancelled".to_string();
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

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }

    pub fn take_needs_full_clear(&mut self) -> bool {
        let needs_clear = self.needs_full_clear;
        self.needs_full_clear = false;
        needs_clear
    }

    pub fn tick(&mut self) {
        if self.task_state.running == Some(crate::tui::task::TuiTaskKind::TestBatch) {
            self.spinner_tick = self.spinner_tick.wrapping_add(1);
        }
    }

    pub fn is_testing_config(&self, config_id: i64) -> bool {
        self.task_state.running == Some(crate::tui::task::TuiTaskKind::TestBatch)
            && self.testing_config_ids.contains(&config_id)
    }
}

#[cfg(test)]
mod tests;
