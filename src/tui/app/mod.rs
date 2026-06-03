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
    ConfigFilter, ConfigListState, ConfigSort, ConfirmKind, ConfirmState, ImportModalState,
    QrModalState, RenameModalState, SourceListState, TestMode, TestScope, TestViewState, TuiAction,
    TuiApp, TuiConfigCommand, TuiView,
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
            TuiAction::RequestDeleteFocused => self.request_delete_focused(),
            TuiAction::RequestPurgeFocused => self.request_purge_focused(),
            TuiAction::RequestDeleteSource => self.request_delete_source(),
            TuiAction::StartTestBatch
            | TuiAction::RuntimeStart
            | TuiAction::RuntimeStop
            | TuiAction::RuntimeRestart
            | TuiAction::RuntimeSwitch
            | TuiAction::RefreshFocusedSource
            | TuiAction::RefreshAllSources
            | TuiAction::OpenQrFocused
            | TuiAction::CopyFocused
            | TuiAction::CopySelected
            | TuiAction::OpenQrApiUrl
            | TuiAction::CopyApiUrl
            | TuiAction::OpenRenameModal
            | TuiAction::RenameSubmit => {}
            TuiAction::OpenImportModal => {
                self.import_modal = Some(crate::tui::app::ImportModalState::default());
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
            TuiAction::ImportInput(ch) => {
                if let Some(modal) = &mut self.import_modal {
                    modal.input.push(ch);
                    modal.error = None;
                }
            }
            TuiAction::ImportBackspace => {
                if let Some(modal) = &mut self.import_modal {
                    modal.input.pop();
                }
            }
            TuiAction::ImportSubmit => {}
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
            }
            TuiAction::SelectFocused
            | TuiAction::EnableFocused
            | TuiAction::DisableFocused
            | TuiAction::ToggleFocused
            | TuiAction::EnableSelected
            | TuiAction::DisableSelected
            | TuiAction::RestoreFocused => {}
            TuiAction::SwitchView(view) => self.switch_view(view),
            TuiAction::None => {}
        }
    }

    pub fn set_status(&mut self, message: impl Into<String>) {
        self.status_message = message.into();
    }
}

#[cfg(test)]
mod tests;
