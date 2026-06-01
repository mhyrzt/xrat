use crossterm::event::{KeyCode, KeyEvent};

use crate::tui::app::TuiAction;

pub fn action_for_confirm_key(key: KeyEvent) -> TuiAction {
    match key.code {
        KeyCode::Enter | KeyCode::Char('y') => TuiAction::Confirm,
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => TuiAction::Cancel,
        _ => TuiAction::None,
    }
}
