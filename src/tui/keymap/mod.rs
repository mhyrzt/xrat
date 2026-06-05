mod chord;
pub use chord::chord_hint;
mod confirm;
mod search;
#[cfg(test)]
mod tests;
mod view;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{TuiAction, TuiView};

#[allow(clippy::too_many_arguments)]
pub fn action_for_key(
    key: KeyEvent,
    active_view: TuiView,
    pending_chord: &mut Option<char>,
    bulk_confirm_open: bool,
    editing_search: bool,
    confirming: bool,
    import_modal_open: bool,
    rename_modal_open: bool,
    qr_modal_open: bool,
) -> TuiAction {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return TuiAction::Quit;
    }

    if qr_modal_open {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q')) {
            return TuiAction::Back;
        }
        return TuiAction::None;
    }

    if import_modal_open {
        return action_for_import_modal_key(key);
    }

    if rename_modal_open {
        return action_for_rename_modal_key(key);
    }

    if bulk_confirm_open {
        return match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => TuiAction::ConfirmBulk,
            _ => TuiAction::CancelBulk,
        };
    }

    if confirming {
        return confirm::action_for_confirm_key(key);
    }

    if editing_search {
        return search::action_for_search_key(key);
    }

    if let Some(leader) = pending_chord.take() {
        return chord::resolve_chord(leader, key.code);
    }

    if active_view == TuiView::Configs
        && let Some(leader) = chord::leader_char(key.code)
    {
        *pending_chord = Some(leader);
        return TuiAction::None;
    }

    view::action_for_view_key(key, active_view)
}

fn action_for_import_modal_key(key: KeyEvent) -> TuiAction {
    match key.code {
        KeyCode::Esc => TuiAction::Back,
        KeyCode::Enter => TuiAction::ImportSubmit,
        KeyCode::Backspace => TuiAction::ImportBackspace,
        KeyCode::Char(ch) => TuiAction::ImportInput(ch),
        _ => TuiAction::None,
    }
}

fn action_for_rename_modal_key(key: KeyEvent) -> TuiAction {
    match key.code {
        KeyCode::Esc => TuiAction::Back,
        KeyCode::Enter => TuiAction::RenameSubmit,
        KeyCode::Backspace => TuiAction::RenameBackspace,
        KeyCode::Char(ch) => TuiAction::RenameInput(ch),
        _ => TuiAction::None,
    }
}
