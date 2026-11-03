use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::TuiAction;

pub fn action_for_search_key(key: KeyEvent) -> TuiAction {
    match key.code {
        KeyCode::Esc => TuiAction::Back,
        KeyCode::Enter => TuiAction::ConfirmSearch,
        KeyCode::Backspace => TuiAction::SearchBackspace,
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            TuiAction::ClearSearch
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            TuiAction::SearchDeleteWord
        }
        KeyCode::Char(ch) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => {
            TuiAction::SearchInput(ch)
        }
        _ => TuiAction::None,
    }
}
