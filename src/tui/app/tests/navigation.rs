use super::helpers::{row, source};
use crate::tui::app::{TuiAction, TuiApp, TuiView};
use crate::tui::data::TuiData;

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
    use crate::tui::app::ConfigSort;

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
fn toggles_deleted_filter_and_resets_focus() {
    let data = TuiData::from_configs(vec![row(1), row(2)]);
    let mut app = TuiApp::with_data(data);
    app.apply(TuiAction::MoveDown);

    app.apply(TuiAction::ToggleDeletedFilter);

    assert!(app.config_list.include_deleted);
    assert_eq!(app.config_list.focused, 0);
}
