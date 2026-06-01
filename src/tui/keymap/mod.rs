mod confirm;
mod search;
mod tests;
mod view;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{TuiAction, TuiView};

pub fn action_for_key(
    key: KeyEvent,
    active_view: TuiView,
    editing_search: bool,
    confirming: bool,
) -> TuiAction {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return TuiAction::Quit;
    }

    if confirming {
        return confirm::action_for_confirm_key(key);
    }

    if editing_search {
        return search::action_for_search_key(key);
    }

    view::action_for_view_key(key, active_view)
}
