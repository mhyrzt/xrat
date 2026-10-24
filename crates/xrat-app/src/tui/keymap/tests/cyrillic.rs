use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::super::action_for_key;
use crate::tui::app::{TestScope, TuiAction, TuiPanel, TuiView};
use crate::tui::keymap::action_for_key_with_import;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
}

fn ctrl_key(ch: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(ch), KeyModifiers::CONTROL)
}

#[test]
fn cyrillic_navigation_and_actions_match_qwerty() {
    let mut pending = None;

    // 'о' should act like 'j' (MoveDown)
    let action = action_for_key(
        key(KeyCode::Char('о')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::MoveDown);

    // 'л' should act like 'k' (MoveUp)
    let action = action_for_key(
        key(KeyCode::Char('л')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::MoveUp);

    // 'й' should act like 'q' (Quit)
    let action = action_for_key(
        key(KeyCode::Char('й')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::Quit);

    // 'б' should act like ',' (OpenSettingsModal)
    let action = action_for_key(
        key(KeyCode::Char('б')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::OpenSettingsModal);

    // Ukrainian 'і' should act like 's' (CycleSort)
    let action = action_for_key(
        key(KeyCode::Char('і')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::CycleSort);
}

#[test]
fn cyrillic_ctrl_shortcuts_work_in_normal_and_text_modes() {
    let mut pending = None;

    // Ctrl+с in normal mode -> Quit
    let action = action_for_key(
        ctrl_key('с'),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::Quit);

    // In search mode: typing regular Russian character enters text
    let action = action_for_key(
        key(KeyCode::Char('щ')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        true, // editing_search
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::SearchInput('щ'));

    // In search mode: Ctrl+ц (Ctrl+w) triggers SearchDeleteWord
    let action = action_for_key(
        ctrl_key('ц'),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        true, // editing_search
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::SearchDeleteWord);

    // In search mode: Ctrl+г (Ctrl+u) triggers ClearSearch
    let action = action_for_key(
        ctrl_key('г'),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        true, // editing_search
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::ClearSearch);
}

#[test]
fn cyrillic_chords_work() {
    let mut pending = None;

    // 'е' acts as 't' (TEST leader)
    let armed = action_for_key_with_import(
        key(KeyCode::Char('е')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
        false,
        None,
    );
    assert_eq!(armed, TuiAction::None);
    assert_eq!(pending, Some('t'));

    // second 'е' acts as 't' (Focused)
    let resolved = action_for_key_with_import(
        key(KeyCode::Char('е')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
        false,
        None,
    );
    assert_eq!(resolved, TuiAction::StartTest(TestScope::Focused));
    assert_eq!(pending, None);
}

#[test]
fn cyrillic_bulk_confirm_works() {
    let mut pending = None;

    // 'н' acts as 'y' (ConfirmBulk)
    let action = action_for_key_with_import(
        key(KeyCode::Char('н')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        true, // bulk_confirm_open
        false,
        false,
        false,
        false,
        false,
        None,
    );
    assert_eq!(action, TuiAction::ConfirmBulk);
}
