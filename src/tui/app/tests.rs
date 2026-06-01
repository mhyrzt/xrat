use crate::tui::data::{TuiConfigRow, TuiData, TuiSourceRow};
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

use super::{ConfigSort, ConfirmKind, TestScope, TuiAction, TuiApp, TuiConfigCommand, TuiView};

#[test]
fn switches_active_view() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::SwitchView(TuiView::Runtime));

    assert_eq!(app.active_view, TuiView::Runtime);
    assert_eq!(app.status_message, "view: runtime");
}

#[test]
fn back_closes_help() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::ShowHelp);
    app.apply(TuiAction::Back);

    assert!(!app.show_help);
    assert_eq!(app.status_message, "ready");
}

#[test]
fn moves_config_focus_within_bounds() {
    let data = TuiData::from_configs(vec![row(1), row(2)]);
    let mut app = TuiApp::with_data(data);

    app.apply(TuiAction::MoveDown);
    app.apply(TuiAction::MoveDown);
    assert_eq!(app.config_list.focused, 1);

    app.apply(TuiAction::MoveUp);
    app.apply(TuiAction::MoveUp);
    assert_eq!(app.config_list.focused, 0);
}

#[test]
fn moves_source_focus_within_bounds() {
    let data = TuiData::from_configs_and_sources(vec![], vec![source(1), source(2)]);
    let mut app = TuiApp::with_data(data);
    app.apply(TuiAction::SwitchView(TuiView::Sources));

    app.apply(TuiAction::MoveDown);
    app.apply(TuiAction::MoveDown);
    assert_eq!(app.source_list.focused, 1);

    app.apply(TuiAction::MoveUp);
    app.apply(TuiAction::MoveUp);
    assert_eq!(app.source_list.focused, 0);
}

#[test]
fn filters_visible_configs_by_search_text() {
    let mut trojan = row(2);
    trojan.name = "fast trojan".to_string();
    trojan.protocol = "trojan".to_string();
    let data = TuiData::from_configs(vec![row(1), trojan]);
    let mut app = TuiApp::with_data(data);

    app.apply(TuiAction::BeginSearch);
    for ch in "trojan".chars() {
        app.apply(TuiAction::SearchInput(ch));
    }

    let visible: Vec<_> = app
        .visible_configs()
        .into_iter()
        .map(|row| row.id)
        .collect();
    assert_eq!(visible, vec![2]);
    assert_eq!(app.focused_config().map(|row| row.id), Some(2));
}

#[test]
fn clearing_search_restores_visible_configs() {
    let data = TuiData::from_configs(vec![row(1), row(2)]);
    let mut app = TuiApp::with_data(data);

    app.apply(TuiAction::BeginSearch);
    app.apply(TuiAction::SearchInput('z'));
    assert!(app.visible_configs().is_empty());

    app.apply(TuiAction::ClearSearch);

    assert_eq!(app.visible_configs().len(), 2);
}

#[test]
fn cycles_config_sort_order() {
    let mut app = TuiApp::with_data(TuiData::from_configs(vec![row(2), row(1)]));

    assert_eq!(app.config_list.sort, ConfigSort::RealDelay);
    app.apply(TuiAction::CycleSort);

    assert_eq!(app.config_list.sort, ConfigSort::Id);
    let visible: Vec<_> = app
        .visible_configs()
        .into_iter()
        .map(|row| row.id)
        .collect();
    assert_eq!(visible, vec![1, 2]);
}

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
fn toggles_deleted_filter_and_resets_focus() {
    let data = TuiData::from_configs(vec![row(1), row(2)]);
    let mut app = TuiApp::with_data(data);
    app.apply(TuiAction::MoveDown);

    app.apply(TuiAction::ToggleDeletedFilter);

    assert!(app.config_list.include_deleted);
    assert_eq!(app.config_list.focused, 0);
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
fn applies_task_completion_and_reloads_data() {
    let mut app = TuiApp::with_data(TuiData::from_configs(vec![row(1)]));

    app.apply_task_event(TuiTaskEvent::Started {
        kind: TuiTaskKind::ReloadData,
    });
    assert_eq!(app.task_state.running, Some(TuiTaskKind::ReloadData));

    app.apply_task_event(TuiTaskEvent::Completed {
        kind: TuiTaskKind::ReloadData,
        message: "reloaded".to_string(),
        data: Some(TuiData::from_configs(vec![row(1), row(2)])),
    });

    assert_eq!(app.task_state.running, None);
    assert_eq!(app.status_message, "reloaded");
    assert_eq!(app.data.total_configs, 2);
}

fn row(id: i64) -> TuiConfigRow {
    TuiConfigRow {
        id,
        name: format!("config-{id}"),
        protocol: "vless".to_string(),
        address: "example.com".to_string(),
        port: 443,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        real_delay_ms: Some(100),
        tcp_ms: Some(20),
        failure_reason: None,
        source_id: None,
        is_active: false,
        is_enabled: true,
        is_selected: false,
        is_deleted: false,
    }
}

fn source(id: i64) -> TuiSourceRow {
    TuiSourceRow {
        id,
        kind: "url".to_string(),
        value: format!("https://example.com/{id}"),
        name: Some(format!("source-{id}")),
        config_count: id,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}
