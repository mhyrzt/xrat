use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::super::{action_for_key, action_for_key_with_import};
use crate::tui::app::{SettingsMode, TuiAction, TuiPanel, TuiView};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

fn import_act(key: KeyEvent) -> TuiAction {
    action_for_key_with_import(
        key,
        TuiView::Configs,
        TuiPanel::Table,
        &mut None,
        false,
        false,
        false,
        true,
        false,
        false,
        None,
    )
}

fn settings_act(key: KeyEvent, mode: SettingsMode) -> TuiAction {
    action_for_key_with_import(
        key,
        TuiView::Configs,
        TuiPanel::Table,
        &mut None,
        false,
        false,
        false,
        false,
        false,
        false,
        Some(mode),
    )
}

#[test]
fn settings_modal_has_isolated_browse_and_edit_keys() {
    assert_eq!(
        settings_act(key(KeyCode::Char('j')), SettingsMode::Browse),
        TuiAction::SettingsMove(1)
    );
    assert_eq!(
        settings_act(key(KeyCode::Enter), SettingsMode::Browse),
        TuiAction::SettingsSubmit
    );
    assert_eq!(
        settings_act(key(KeyCode::Char('j')), SettingsMode::Edit),
        TuiAction::SettingsInput('j')
    );
    assert_eq!(
        settings_act(key(KeyCode::Esc), SettingsMode::Edit),
        TuiAction::Back
    );
    assert_eq!(
        settings_act(key(KeyCode::Right), SettingsMode::Browse),
        TuiAction::SettingsFocusFields
    );
    assert_eq!(
        settings_act(key(KeyCode::Left), SettingsMode::Browse),
        TuiAction::SettingsFocusSections
    );
}

#[test]
fn settings_shortcuts_do_not_leak_modified_characters_into_input() {
    let control_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
    let control_x = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);

    assert_eq!(
        settings_act(control_s, SettingsMode::Edit),
        TuiAction::SettingsSave
    );
    assert_eq!(
        settings_act(control_s, SettingsMode::Search),
        TuiAction::SettingsSave
    );
    assert_eq!(settings_act(control_x, SettingsMode::Edit), TuiAction::None);
}

#[test]
fn global_confirmation_takes_priority_over_open_settings() {
    assert_eq!(
        action_for_key_with_import(
            key(KeyCode::Char('y')),
            TuiView::Configs,
            TuiPanel::Table,
            &mut None,
            false,
            false,
            true,
            false,
            false,
            false,
            Some(SettingsMode::Browse),
        ),
        TuiAction::Confirm
    );
}

#[test]
fn settings_discard_confirmation_accepts_only_yes_or_no() {
    assert_eq!(
        settings_act(key(KeyCode::Char('y')), SettingsMode::DiscardConfirm),
        TuiAction::SettingsConfirmDiscard(true)
    );
    assert_eq!(
        settings_act(key(KeyCode::Char('n')), SettingsMode::DiscardConfirm),
        TuiAction::SettingsConfirmDiscard(false)
    );
}

/// Wrapper preserving the pre-chord call shape: no armed chord, no bulk confirm.
#[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
fn act(
    key: KeyEvent,
    view: TuiView,
    editing_search: bool,
    confirming: bool,
    rename_modal_open: bool,
    qr_modal_open: bool,
) -> TuiAction {
    action_for_key(
        key,
        view,
        TuiPanel::Table,
        &mut None,
        false,
        editing_search,
        confirming,
        rename_modal_open,
        qr_modal_open,
    )
}

#[test]
fn maps_confirm_keys() {
    assert_eq!(
        act(
            key(KeyCode::Enter),
            TuiView::Configs,
            false,
            true,
            false,
            false
        ),
        TuiAction::Confirm
    );
    assert_eq!(
        act(
            key(KeyCode::Char('y')),
            TuiView::Configs,
            false,
            true,
            false,
            false
        ),
        TuiAction::Confirm
    );
    assert_eq!(
        act(
            key(KeyCode::Char('n')),
            TuiView::Configs,
            false,
            true,
            false,
            false
        ),
        TuiAction::Cancel
    );
}

#[test]
fn maps_search_editing_keys() {
    assert_eq!(
        act(
            key(KeyCode::Char('v')),
            TuiView::Configs,
            true,
            false,
            false,
            false
        ),
        TuiAction::SearchInput('v')
    );
    assert_eq!(
        act(
            key(KeyCode::Backspace),
            TuiView::Configs,
            true,
            false,
            false,
            false
        ),
        TuiAction::SearchBackspace
    );
    assert_eq!(
        act(
            key(KeyCode::Enter),
            TuiView::Configs,
            true,
            false,
            false,
            false
        ),
        TuiAction::ConfirmSearch
    );
    assert_eq!(
        act(
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            TuiView::Configs,
            true,
            false,
            false,
            false
        ),
        TuiAction::ClearSearch
    );
}

#[test]
fn import_modal_consumes_edit_and_submit_keys() {
    assert_eq!(
        import_act(key(KeyCode::Char('v'))),
        TuiAction::ImportInput('v')
    );
    assert_eq!(
        import_act(key(KeyCode::Backspace)),
        TuiAction::ImportBackspace
    );
    assert_eq!(import_act(key(KeyCode::Enter)), TuiAction::ImportSubmit);
    assert_eq!(import_act(key(KeyCode::Esc)), TuiAction::Back);
}
