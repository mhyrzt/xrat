use super::helpers::{row, source};
use crate::tui::app::{TuiAction, TuiApp, TuiConfigCommand, TuiView};
use crate::tui::data::TuiData;

#[test]
fn maps_focused_config_actions_to_commands() {
    let data = TuiData::from_configs(vec![row(1)]);
    let app = TuiApp::with_data(data);

    assert_eq!(
        app.config_command_for_action(TuiAction::SelectFocused),
        Some(TuiConfigCommand::Select(1))
    );
    assert_eq!(
        app.config_command_for_action(TuiAction::EnableFocused),
        Some(TuiConfigCommand::Enable(1))
    );
    assert_eq!(
        app.config_command_for_action(TuiAction::DisableFocused),
        Some(TuiConfigCommand::Disable(1))
    );
}

#[test]
fn opens_and_cancels_delete_confirmation() {
    use crate::tui::app::ConfirmKind;

    let data = TuiData::from_configs(vec![row(1)]);
    let mut app = TuiApp::with_data(data);

    app.apply(TuiAction::RequestDeleteFocused);

    assert_eq!(
        app.confirm.as_ref().map(|confirm| confirm.kind),
        Some(ConfirmKind::SoftDeleteConfig(1))
    );
    assert_eq!(
        app.pending_confirm_command(),
        Some(TuiConfigCommand::SoftDelete(1))
    );

    app.apply(TuiAction::Cancel);

    assert!(app.confirm.is_none());
    assert_eq!(app.status_message, "cancelled");
}

#[test]
fn restore_command_only_applies_to_deleted_configs() {
    let mut deleted = row(1);
    deleted.is_deleted = true;
    let data = TuiData::from_configs(vec![deleted]);
    let app = TuiApp::with_data(data);

    assert_eq!(
        app.config_command_for_action(TuiAction::RestoreFocused),
        Some(TuiConfigCommand::Restore(1))
    );
}

#[test]
fn counts_current_test_scope() {
    use crate::tui::app::TestScope;

    let mut selected = row(2);
    selected.is_selected = true;
    let mut failed = row(3);
    failed.failure_reason = Some("timeout".to_string());
    let data = TuiData::from_configs(vec![row(1), selected, failed]);
    let mut app = TuiApp::with_data(data);

    assert_eq!(app.test_scope_count(), 3);

    app.test_state.scope = TestScope::Selected;
    assert_eq!(app.test_scope_count(), 1);

    app.test_state.scope = TestScope::Failed;
    assert_eq!(app.test_scope_count(), 1);
}

#[test]
fn collects_config_ids_for_current_test_scope() {
    use crate::tui::app::TestScope;

    let mut selected = row(2);
    selected.is_selected = true;
    let mut failed = row(3);
    failed.failure_reason = Some("timeout".to_string());
    let data = TuiData::from_configs(vec![row(1), selected, failed]);
    let mut app = TuiApp::with_data(data);

    assert_eq!(app.test_config_ids(), vec![1, 2, 3]);

    app.test_state.scope = TestScope::Selected;
    assert_eq!(app.test_config_ids(), vec![2]);

    app.test_state.scope = TestScope::Failed;
    assert_eq!(app.test_config_ids(), vec![3]);
}

#[test]
fn test_scope_shortcuts_update_scope() {
    use crate::tui::app::TestScope;

    let data = TuiData::from_configs(vec![row(1)]);
    let mut app = TuiApp::with_data(data);

    app.test_state.scope = TestScope::Focused;
    app.apply(TuiAction::StartTestAllEnabled);
    assert_eq!(app.test_state.scope, TestScope::AllEnabled);

    app.apply(TuiAction::StartTestFiltered);
    assert_eq!(app.test_state.scope, TestScope::Filtered);
}

#[test]
fn refresh_focused_source_action_targets_correct_source() {
    let data = TuiData::from_configs_and_sources(vec![], vec![source(1), source(2)]);
    let mut app = TuiApp::with_data(data);
    app.apply(TuiAction::SwitchView(TuiView::Sources));

    // simulate the capture logic from run/mod.rs
    let focused = app
        .focused_source()
        .map(|s| (s.id, s.value.clone()))
        .filter(|(_, v)| !v.is_empty());
    assert_eq!(focused.as_ref().map(|(id, _)| *id), Some(1));

    app.apply(TuiAction::MoveDown);
    let focused = app
        .focused_source()
        .map(|s| (s.id, s.value.clone()))
        .filter(|(_, v)| !v.is_empty());
    assert_eq!(focused.as_ref().map(|(id, _)| *id), Some(2));
}

#[test]
fn focused_source_returns_current_source() {
    let data = TuiData::from_configs_and_sources(vec![], vec![source(1), source(2)]);
    let mut app = TuiApp::with_data(data);
    app.apply(TuiAction::SwitchView(TuiView::Sources));

    assert_eq!(app.focused_source().map(|s| s.id), Some(1));

    app.apply(TuiAction::MoveDown);
    assert_eq!(app.focused_source().map(|s| s.id), Some(2));

    app.apply(TuiAction::MoveDown);
    assert_eq!(app.focused_source().map(|s| s.id), Some(2));
}

#[test]
fn selected_configs_scope_for_copy() {
    let mut sel_a = row(1);
    sel_a.is_selected = true;
    let mut sel_b = row(2);
    sel_b.is_selected = true;
    let unsel = row(3);

    let data = TuiData::from_configs(vec![sel_a, sel_b, unsel]);
    let app = TuiApp::with_data(data);

    let selected_ids: Vec<i64> = app
        .data
        .configs
        .iter()
        .filter(|c| c.is_selected)
        .map(|c| c.id)
        .collect();
    assert_eq!(selected_ids, vec![1, 2]);
    assert!(!selected_ids.contains(&3));
}
