mod chord;
pub use chord::{chord_entries, chord_title};
mod confirm;
mod search;
#[cfg(test)]
mod tests;
mod view;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{SettingsMode, TuiAction, TuiPanel, TuiView};

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub fn action_for_key(
    key: KeyEvent,
    active_view: TuiView,
    focused_panel: TuiPanel,
    pending_chord: &mut Option<char>,
    bulk_confirm_open: bool,
    editing_search: bool,
    confirming: bool,
    rename_modal_open: bool,
    qr_modal_open: bool,
) -> TuiAction {
    action_for_key_with_import(
        key,
        active_view,
        focused_panel,
        pending_chord,
        bulk_confirm_open,
        editing_search,
        confirming,
        false,
        rename_modal_open,
        qr_modal_open,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn action_for_key_with_import(
    key: KeyEvent,
    active_view: TuiView,
    focused_panel: TuiPanel,
    pending_chord: &mut Option<char>,
    bulk_confirm_open: bool,
    editing_search: bool,
    confirming: bool,
    import_modal_open: bool,
    rename_modal_open: bool,
    qr_modal_open: bool,
    settings_mode: Option<SettingsMode>,
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

    if let Some(mode) = settings_mode {
        return action_for_settings_modal_key(key, mode);
    }

    if editing_search {
        return search::action_for_search_key(key);
    }

    if let Some(leader) = pending_chord.take() {
        return chord::resolve_chord(leader, key.code);
    }

    if key.code == KeyCode::Char('u') {
        return TuiAction::RefreshAllSources;
    }

    if let Some(leader) = chord::leader_char(key.code)
        && (active_view == TuiView::Configs || leader == 'a')
    {
        *pending_chord = Some(leader);
        return TuiAction::None;
    }

    view::action_for_view_key(key, active_view, focused_panel)
}

fn action_for_settings_modal_key(key: KeyEvent, mode: SettingsMode) -> TuiAction {
    if mode == SettingsMode::DiscardConfirm {
        return match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => TuiAction::SettingsConfirmDiscard(true),
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                TuiAction::SettingsConfirmDiscard(false)
            }
            _ => TuiAction::None,
        };
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('u') {
        return TuiAction::SettingsClearInput;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('s') {
        return TuiAction::SettingsSave;
    }

    match mode {
        SettingsMode::Search | SettingsMode::Edit => match key.code {
            KeyCode::Esc => TuiAction::Back,
            KeyCode::Enter => TuiAction::SettingsSubmit,
            KeyCode::Backspace => TuiAction::SettingsBackspace,
            KeyCode::Char(ch)
                if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT =>
            {
                TuiAction::SettingsInput(ch)
            }
            _ => TuiAction::None,
        },
        SettingsMode::Browse => match key.code {
            KeyCode::Esc => TuiAction::Back,
            KeyCode::Tab | KeyCode::BackTab => TuiAction::SettingsSwitchPane,
            KeyCode::Char('j') | KeyCode::Down => TuiAction::SettingsMove(1),
            KeyCode::Char('k') | KeyCode::Up => TuiAction::SettingsMove(-1),
            KeyCode::Char('/') => TuiAction::SettingsBeginSearch,
            KeyCode::Enter | KeyCode::Char(' ') => TuiAction::SettingsSubmit,
            KeyCode::Char('r') => TuiAction::SettingsReset,
            KeyCode::Left => TuiAction::SettingsFocusSections,
            KeyCode::Right => TuiAction::SettingsFocusFields,
            KeyCode::Char('h') => TuiAction::SettingsCycle(-1),
            KeyCode::Char('l') => TuiAction::SettingsCycle(1),
            _ => TuiAction::None,
        },
        SettingsMode::DiscardConfirm => TuiAction::None,
    }
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
