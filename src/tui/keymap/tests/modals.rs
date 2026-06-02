use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::super::action_for_key;
use crate::tui::app::{TuiAction, TuiView};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

#[test]
fn maps_confirm_keys() {
    assert_eq!(
        action_for_key(
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
        action_for_key(
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
        action_for_key(
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
        action_for_key(
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
        action_for_key(
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
        action_for_key(
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
        action_for_key(
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
fn maps_tests_view_actions() {
    assert_eq!(
        action_for_key(
            key(KeyCode::Char('s')),
            TuiView::Tests,
            false,
            false,
            false,
            false
        ),
        TuiAction::StartTestBatch
    );
    assert_eq!(
        action_for_key(
            key(KeyCode::Char('c')),
            TuiView::Tests,
            false,
            false,
            false,
            false
        ),
        TuiAction::CancelTestBatch
    );
}
