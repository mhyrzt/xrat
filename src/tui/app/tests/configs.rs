use super::helpers::row;
use crate::tui::app::{TuiAction, TuiApp, TuiConfigCommand};
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
