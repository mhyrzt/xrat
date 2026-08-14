use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::super::{action_for_key, action_for_key_with_import};
use crate::tui::app::{TuiAction, TuiPanel, TuiView};

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
    )
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
