use super::helpers::{row, source};
use crate::tui::app::{TuiAction, TuiApp, TuiConfigCommand};
use crate::tui::data::TuiData;

#[test]
fn maps_focused_config_actions_to_commands() {
    let data = TuiData::from_configs(vec![row(1)]);
    let app = TuiApp::with_data(data);

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
    assert!(app.needs_full_clear);
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

    let mut failed = row(3);
    failed.failure_reason = Some("timeout".to_string());
    let data = TuiData::from_configs(vec![row(1), row(2), failed]);
    let mut app = TuiApp::with_data(data);

    assert_eq!(app.test_scope_count(), 3);

    app.test_state.scope = TestScope::Failed;
    assert_eq!(app.test_scope_count(), 1);
}

#[test]
fn collects_config_ids_for_current_test_scope() {
    use crate::tui::app::TestScope;

    let mut failed = row(3);
    failed.failure_reason = Some("timeout".to_string());
    let data = TuiData::from_configs(vec![row(1), row(2), failed]);
    let mut app = TuiApp::with_data(data);

    assert_eq!(app.test_config_ids(), vec![1, 2, 3]);

    app.test_state.scope = TestScope::Failed;
    assert_eq!(app.test_config_ids(), vec![3]);
}

#[test]
fn test_scope_shortcuts_update_scope() {
    use crate::tui::app::TestScope;

    let data = TuiData::from_configs(vec![row(1)]);
    let mut app = TuiApp::with_data(data);

    app.test_state.scope = TestScope::Focused;
    app.apply(TuiAction::StartTest(TestScope::AllEnabled));
    assert_eq!(app.test_state.scope, TestScope::AllEnabled);

    app.apply(TuiAction::StartTest(TestScope::Filtered));
    assert_eq!(app.test_state.scope, TestScope::Filtered);
}

#[test]
fn collects_config_ids_for_bulk_scopes() {
    use crate::tui::app::BulkOp;

    let mut failed = row(2);
    failed.failure_reason = Some("timeout".to_string());
    let mut disabled = row(3);
    disabled.is_enabled = false;
    let mut deleted = row(4);
    deleted.is_deleted = true;

    let data = TuiData::from_configs(vec![row(1), failed, disabled, deleted]);
    let app = TuiApp::with_data(data);

    assert_eq!(app.bulk_config_ids(BulkOp::DeleteFailed), vec![2]);
    assert_eq!(app.bulk_config_ids(BulkOp::PurgeFailed), vec![2]);
    assert_eq!(app.bulk_config_ids(BulkOp::DeleteDisabled), vec![3]);
    assert_eq!(app.bulk_config_ids(BulkOp::PurgeAllDeleted), vec![4]);
    assert_eq!(app.bulk_config_ids(BulkOp::RestoreAllDeleted), vec![4]);
}

#[test]
fn request_bulk_arms_confirm_or_reports_empty() {
    use crate::tui::app::BulkOp;

    let mut deleted = row(2);
    deleted.is_deleted = true;
    let data = TuiData::from_configs(vec![row(1), deleted]);
    let mut app = TuiApp::with_data(data);

    app.apply(TuiAction::RequestBulk(BulkOp::PurgeAllDeleted));
    assert_eq!(app.pending_bulk, Some(BulkOp::PurgeAllDeleted));

    app.apply(TuiAction::ConfirmBulk);
    assert!(app.pending_bulk.is_none());

    app.apply(TuiAction::RequestBulk(BulkOp::DeleteFailed));
    assert!(app.pending_bulk.is_none());
}

#[test]
fn refresh_focused_source_action_targets_correct_source() {
    let data = TuiData::from_configs_and_sources(vec![], vec![source(1), source(2)]);
    let mut app = TuiApp::with_data(data);
    app.apply(TuiAction::NextTab);

    // indices 0 and 1 are "All" and "Orphans"; move twice to the first source.
    app.apply(TuiAction::MoveDown);
    app.apply(TuiAction::MoveDown);

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
    app.apply(TuiAction::NextTab);

    // indices 0 and 1 are the synthetic "All" and "Orphans" rows.
    assert_eq!(app.focused_source().map(|s| s.id), None);

    app.apply(TuiAction::MoveDown);
    assert_eq!(app.focused_source().map(|s| s.id), None);

    app.apply(TuiAction::MoveDown);
    assert_eq!(app.focused_source().map(|s| s.id), Some(1));

    app.apply(TuiAction::MoveDown);
    assert_eq!(app.focused_source().map(|s| s.id), Some(2));

    app.apply(TuiAction::MoveDown);
    assert_eq!(app.focused_source().map(|s| s.id), Some(2));
}
