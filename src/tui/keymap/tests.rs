use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::app::{TuiAction, TuiView};

use super::action_for_key;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

#[test]
fn maps_global_quit_keys() {
    assert_eq!(
        action_for_key(key(KeyCode::Char('q')), TuiView::Configs, false, false),
        TuiAction::Quit
    );
    assert_eq!(
        action_for_key(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            TuiView::Configs,
            false,
            false
        ),
        TuiAction::Quit
    );
}

#[test]
fn maps_view_switching_keys() {
    assert_eq!(
        action_for_key(key(KeyCode::Char('1')), TuiView::Configs, false, false),
        TuiAction::SwitchView(TuiView::Configs)
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('4')), TuiView::Configs, false, false),
        TuiAction::SwitchView(TuiView::Runtime)
    );
}

#[test]
fn maps_navigation_and_help_keys() {
    assert_eq!(
        action_for_key(key(KeyCode::Down), TuiView::Configs, false, false),
        TuiAction::MoveDown
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('k')), TuiView::Configs, false, false),
        TuiAction::MoveUp
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('?')), TuiView::Configs, false, false),
        TuiAction::ShowHelp
    );
    assert_eq!(
        action_for_key(key(KeyCode::Esc), TuiView::Configs, false, false),
        TuiAction::Back
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('/')), TuiView::Configs, false, false),
        TuiAction::BeginSearch
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('s')), TuiView::Configs, false, false),
        TuiAction::CycleSort
    );
}

#[test]
fn maps_config_action_keys() {
    assert_eq!(
        action_for_key(key(KeyCode::Char(' ')), TuiView::Configs, false, false),
        TuiAction::SelectFocused
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('e')), TuiView::Configs, false, false),
        TuiAction::EnableFocused
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('x')), TuiView::Configs, false, false),
        TuiAction::DisableFocused
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('d')), TuiView::Configs, false, false),
        TuiAction::RequestDeleteFocused
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('D')), TuiView::Configs, false, false),
        TuiAction::RequestPurgeFocused
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('r')), TuiView::Configs, false, false),
        TuiAction::RestoreFocused
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('f')), TuiView::Configs, false, false),
        TuiAction::ToggleDeletedFilter
    );
}

#[test]
fn maps_confirm_keys() {
    assert_eq!(
        action_for_key(key(KeyCode::Enter), TuiView::Configs, false, true),
        TuiAction::Confirm
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('y')), TuiView::Configs, false, true),
        TuiAction::Confirm
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('n')), TuiView::Configs, false, true),
        TuiAction::Cancel
    );
}

#[test]
fn maps_search_editing_keys() {
    assert_eq!(
        action_for_key(key(KeyCode::Char('v')), TuiView::Configs, true, false),
        TuiAction::SearchInput('v')
    );
    assert_eq!(
        action_for_key(key(KeyCode::Backspace), TuiView::Configs, true, false),
        TuiAction::SearchBackspace
    );
    assert_eq!(
        action_for_key(key(KeyCode::Enter), TuiView::Configs, true, false),
        TuiAction::ConfirmSearch
    );
    assert_eq!(
        action_for_key(
            KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL),
            TuiView::Configs,
            true,
            false
        ),
        TuiAction::ClearSearch
    );
}

#[test]
fn maps_tests_view_actions() {
    assert_eq!(
        action_for_key(key(KeyCode::Char('s')), TuiView::Tests, false, false),
        TuiAction::StartTestBatch
    );
    assert_eq!(
        action_for_key(key(KeyCode::Char('c')), TuiView::Tests, false, false),
        TuiAction::CancelTestBatch
    );
}
