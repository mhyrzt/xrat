use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::super::action_for_key;
use crate::tui::app::{TuiAction, TuiPanel, TuiView};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::empty())
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

/// Drive a two-key chord: arm `leader`, then resolve with `second`.
fn chord(leader: KeyCode, second: KeyCode, view: TuiView) -> TuiAction {
    let mut pending = None;
    let armed = action_for_key(
        key(leader),
        view,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(armed, TuiAction::None);
    assert!(pending.is_some());
    action_for_key(
        key(second),
        view,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    )
}

#[test]
fn maps_global_quit_keys() {
    assert_eq!(
        act(
            key(KeyCode::Char('q')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::Quit
    );
    assert_eq!(
        act(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::Quit
    );
}

#[test]
fn maps_view_switching_keys() {
    assert_eq!(
        act(
            key(KeyCode::Char('[')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::PrevTab
    );
    assert_eq!(
        act(
            key(KeyCode::Char(']')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::NextTab
    );
}

#[test]
fn maps_tab_to_panel_focus() {
    assert_eq!(
        act(
            key(KeyCode::Tab),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::FocusNextPanel
    );
    assert_eq!(
        act(
            key(KeyCode::BackTab),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::FocusPrevPanel
    );
}

#[test]
fn maps_log_tab_keys_only_when_log_panel_focused() {
    assert_eq!(
        action_for_key(
            key(KeyCode::Right),
            TuiView::Configs,
            TuiPanel::Log,
            &mut None,
            false,
            false,
            false,
            false,
            false,
        ),
        TuiAction::NextLogTab
    );
    assert_eq!(
        action_for_key(
            key(KeyCode::Left),
            TuiView::Configs,
            TuiPanel::Log,
            &mut None,
            false,
            false,
            false,
            false,
            false,
        ),
        TuiAction::PrevLogTab
    );
    assert_eq!(
        action_for_key(
            key(KeyCode::Right),
            TuiView::Configs,
            TuiPanel::Table,
            &mut None,
            false,
            false,
            false,
            false,
            false,
        ),
        TuiAction::None
    );
}

#[test]
fn maps_navigation_and_help_keys() {
    assert_eq!(
        act(
            key(KeyCode::Down),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::MoveDown
    );
    assert_eq!(
        act(
            key(KeyCode::Char('k')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::MoveUp
    );
    assert_eq!(
        act(
            key(KeyCode::Char('?')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::ShowHelp
    );
    assert_eq!(
        act(
            key(KeyCode::Esc),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::Back
    );
    assert_eq!(
        act(
            key(KeyCode::Char('/')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::BeginSearch
    );
    assert_eq!(
        act(
            key(KeyCode::Char('S')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::CycleSort
    );
}

#[test]
fn maps_enter_start_in_configs() {
    assert_eq!(
        act(
            key(KeyCode::Enter),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::StartFocused
    );
}

#[test]
fn test_chords_resolve_to_scopes() {
    use crate::tui::app::TestScope;
    assert_eq!(
        chord(KeyCode::Char('t'), KeyCode::Char('t'), TuiView::Configs),
        TuiAction::StartTest(TestScope::Focused)
    );
    assert_eq!(
        chord(KeyCode::Char('t'), KeyCode::Char('a'), TuiView::Configs),
        TuiAction::StartTest(TestScope::AllEnabled)
    );
    assert_eq!(
        chord(KeyCode::Char('t'), KeyCode::Char('v'), TuiView::Configs),
        TuiAction::StartTest(TestScope::Filtered)
    );
    assert_eq!(
        chord(KeyCode::Char('t'), KeyCode::Char('r'), TuiView::Configs),
        TuiAction::StartTest(TestScope::Failed)
    );
    assert_eq!(
        chord(KeyCode::Char('t'), KeyCode::Char('s'), TuiView::Configs),
        TuiAction::StartTest(TestScope::Stale)
    );
    assert_eq!(
        chord(KeyCode::Char('t'), KeyCode::Char('c'), TuiView::Configs),
        TuiAction::CancelTestBatch
    );
}

#[test]
fn bare_freed_keys_are_inert_in_configs() {
    for code in [KeyCode::Char('a'), KeyCode::Char('v'), KeyCode::Char('C')] {
        assert_eq!(
            act(key(code), TuiView::Configs, false, false, false, false),
            TuiAction::None
        );
    }
}

#[test]
fn delete_purge_restore_chords_resolve() {
    use crate::tui::app::BulkOp;
    assert_eq!(
        chord(KeyCode::Char('d'), KeyCode::Char('d'), TuiView::Configs),
        TuiAction::RequestDeleteFocused
    );
    assert_eq!(
        chord(KeyCode::Char('d'), KeyCode::Char('f'), TuiView::Configs),
        TuiAction::RequestBulk(BulkOp::DeleteFailed)
    );
    assert_eq!(
        chord(KeyCode::Char('d'), KeyCode::Char('x'), TuiView::Configs),
        TuiAction::RequestBulk(BulkOp::DeleteDisabled)
    );
    assert_eq!(
        chord(KeyCode::Char('D'), KeyCode::Char('D'), TuiView::Configs),
        TuiAction::RequestPurgeFocused
    );
    assert_eq!(
        chord(KeyCode::Char('D'), KeyCode::Char('a'), TuiView::Configs),
        TuiAction::RequestBulk(BulkOp::PurgeAllDeleted)
    );
    assert_eq!(
        chord(KeyCode::Char('r'), KeyCode::Char('r'), TuiView::Configs),
        TuiAction::RestoreFocused
    );
    assert_eq!(
        chord(KeyCode::Char('r'), KeyCode::Char('a'), TuiView::Configs),
        TuiAction::RequestBulk(BulkOp::RestoreAllDeleted)
    );
}

#[test]
fn unknown_chord_second_key_clears() {
    let mut pending = None;
    action_for_key(
        key(KeyCode::Char('t')),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert!(pending.is_some());
    let resolved = action_for_key(
        key(KeyCode::Esc),
        TuiView::Configs,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(resolved, TuiAction::None);
    assert!(pending.is_none());
}

#[test]
fn chord_leaders_inert_in_sources() {
    let mut pending = None;
    let action = action_for_key(
        key(KeyCode::Char('r')),
        TuiView::Sources,
        TuiPanel::Table,
        &mut pending,
        false,
        false,
        false,
        false,
        false,
    );
    assert_eq!(action, TuiAction::RefreshFocusedSource);
    assert!(pending.is_none());
}

#[test]
fn maps_config_action_keys() {
    assert_eq!(
        act(
            key(KeyCode::Char('e')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::EnableFocused
    );
    assert_eq!(
        act(
            key(KeyCode::Char('x')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::DisableFocused
    );
    assert_eq!(
        act(
            key(KeyCode::Char('K')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::RuntimeStop
    );
    assert_eq!(
        act(
            key(KeyCode::Char('T')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::ToggleDeletedFilter
    );
}

#[test]
fn maps_cycle_filter_key() {
    assert_eq!(
        act(
            key(KeyCode::Char('F')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::CycleFilter
    );
}

#[test]
fn maps_qr_and_copy_keys() {
    assert_eq!(
        act(
            key(KeyCode::Char('y')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::OpenQrFocused
    );
    assert_eq!(
        act(
            key(KeyCode::Char('c')),
            TuiView::Configs,
            false,
            false,
            false,
            false
        ),
        TuiAction::CopyFocused
    );
}

#[test]
fn qr_modal_open_consumes_keys() {
    assert_eq!(
        act(
            key(KeyCode::Esc),
            TuiView::Configs,
            false,
            false,
            false,
            true
        ),
        TuiAction::Back
    );
    assert_eq!(
        act(
            key(KeyCode::Char('q')),
            TuiView::Configs,
            false,
            false,
            false,
            true
        ),
        TuiAction::Back
    );
    assert_eq!(
        act(
            key(KeyCode::Char('s')),
            TuiView::Configs,
            false,
            false,
            false,
            true
        ),
        TuiAction::None
    );
}

#[test]
fn maps_sources_view_actions() {
    assert_eq!(
        act(
            key(KeyCode::Char('r')),
            TuiView::Sources,
            false,
            false,
            false,
            false
        ),
        TuiAction::RefreshFocusedSource
    );
    assert_eq!(
        act(
            key(KeyCode::Char('R')),
            TuiView::Sources,
            false,
            false,
            false,
            false
        ),
        TuiAction::RefreshAllSources
    );
}
