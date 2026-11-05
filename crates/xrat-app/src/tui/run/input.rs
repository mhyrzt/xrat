use crossterm::event::{KeyCode, KeyModifiers};

use crate::tui::app::{SettingsMode, SplitModalMode, TuiAction, TuiApp};
use crate::tui::keymap::TuiKeyEvent;

pub(super) fn apply_key_text(app: &mut TuiApp, key: &TuiKeyEvent) -> bool {
    let Some(text) = key.text.as_deref().filter(|text| !text.is_empty()) else {
        return false;
    };
    if !matches!(key.code, KeyCode::Char(_)) {
        return false;
    }
    if key.modifiers.contains(KeyModifiers::SUPER)
        || (key.modifiers.contains(KeyModifiers::CONTROL)
            && !key.modifiers.contains(KeyModifiers::ALT))
    {
        return false;
    }
    let action: fn(char) -> TuiAction = if let Some(modal) = &app.split_modal {
        if !matches!(
            modal.mode(),
            SplitModalMode::AddInput | SplitModalMode::Search
        ) {
            return false;
        }
        TuiAction::SplitInput
    } else if app.qr_modal.is_some() {
        return false;
    } else if app.import_modal.is_some() {
        TuiAction::ImportInput
    } else if app.rename_modal.is_some() {
        TuiAction::RenameInput
    } else if app.pending_bulk.is_some() || app.confirm.is_some() {
        return false;
    } else if let Some(modal) = &app.settings_modal {
        if !matches!(modal.mode(), SettingsMode::Edit | SettingsMode::Search) {
            return false;
        }
        TuiAction::SettingsInput
    } else if app.config_list.editing_search {
        TuiAction::SearchInput
    } else {
        return false;
    };
    for character in text.chars().filter(|character| !character.is_control()) {
        app.apply(action(character));
    }
    true
}

#[cfg(test)]
mod tests;
