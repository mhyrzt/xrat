use super::helpers::{row, source};
use crate::tui::app::{TuiAction, TuiApp, TuiPanel};
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
fn jumps_config_focus_to_top_and_bottom() {
    let data = TuiData::from_configs((1..=6).map(row).collect());
    let mut app = TuiApp::with_data(data);

    app.apply(TuiAction::MoveBottom);
    assert_eq!(app.config_list.focused, 5);

    app.apply(TuiAction::MoveTop);
    assert_eq!(app.config_list.focused, 0);
}

#[test]
fn pages_config_focus_by_viewport_height() {
    let data = TuiData::from_configs((1..=10).map(row).collect());
    let mut app = TuiApp::with_data(data);
    app.panel_viewport.table.set(3);

    app.apply(TuiAction::PageDown);
    assert_eq!(app.config_list.focused, 3);

    app.apply(TuiAction::PageDown);
    assert_eq!(app.config_list.focused, 6);

    app.apply(TuiAction::PageUp);
    assert_eq!(app.config_list.focused, 3);
}

#[test]
fn paging_clamps_at_config_bounds() {
    let data = TuiData::from_configs((1..=4).map(row).collect());
    let mut app = TuiApp::with_data(data);
    app.panel_viewport.table.set(10);

    app.apply(TuiAction::PageDown);
    assert_eq!(app.config_list.focused, 3);

    app.apply(TuiAction::PageUp);
    assert_eq!(app.config_list.focused, 0);
}

#[test]
fn jumps_scroll_panel_to_top() {
    let mut app = TuiApp::default();
    app.apply(TuiAction::FocusPanel(TuiPanel::Log));
    app.panel_scroll.log.set(7);

    app.apply(TuiAction::MoveTop);
    assert_eq!(app.panel_scroll.log.get(), 0);
}

#[test]
fn focuses_panel_directly() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::FocusPanel(TuiPanel::Log));
    assert_eq!(app.focused_panel, TuiPanel::Log);

    app.apply(TuiAction::FocusPanel(TuiPanel::Detail));
    assert_eq!(app.focused_panel, TuiPanel::Detail);

    app.apply(TuiAction::FocusPanel(TuiPanel::Runtime));
    assert_eq!(app.focused_panel, TuiPanel::Runtime);

    app.apply(TuiAction::FocusPanel(TuiPanel::Table));
    assert_eq!(app.focused_panel, TuiPanel::Table);
}

#[test]
fn moves_source_focus_within_bounds() {
    let data = TuiData::from_configs_and_sources(vec![], vec![source(1), source(2)]);
    let mut app = TuiApp::with_data(data);
    app.apply(TuiAction::NextTab);

    // indices 0 and 1 are the "All" and "Orphans" rows; sources start at 2.
    app.apply(TuiAction::MoveDown);
    app.apply(TuiAction::MoveDown);
    assert_eq!(app.source_list.focused, 2);

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
    assert_eq!(app.config_list.sort, ConfigSort::TcpDelay);
    app.apply(TuiAction::CycleSort);
    assert_eq!(app.config_list.sort, ConfigSort::Id);
    let visible: Vec<_> = app
        .visible_configs()
        .into_iter()
        .map(|row| row.id)
        .collect();
    assert_eq!(visible, vec![1, 2]);

    app.apply(TuiAction::CycleSort); // Name
    app.apply(TuiAction::CycleSort); // Protocol
    app.apply(TuiAction::CycleSort); // Source
    app.apply(TuiAction::CycleSort);
    assert_eq!(app.config_list.sort, ConfigSort::LastTested);
    app.apply(TuiAction::CycleSort);
    assert_eq!(app.config_list.sort, ConfigSort::ImportedAt);
    app.apply(TuiAction::CycleSort);
    assert_eq!(app.config_list.sort, ConfigSort::RealDelay);
}

#[test]
fn cycles_protocol_filter() {
    let mut vless = row(1);
    vless.protocol = "vless".to_string();

    let mut trojan = row(2);
    trojan.protocol = "trojan".to_string();

    let mut vless2 = row(3);
    vless2.protocol = "vless".to_string();

    let data = TuiData::from_configs(vec![vless, trojan, vless2]);
    let mut app = TuiApp::with_data(data);

    assert!(app.config_list.protocol_filter.is_none());
    assert_eq!(app.visible_configs().len(), 3);

    app.apply(TuiAction::CycleProtocolFilter);
    assert_eq!(app.config_list.protocol_filter.as_deref(), Some("trojan"));
    let visible: Vec<i64> = app.visible_configs().iter().map(|r| r.id).collect();
    assert_eq!(visible, vec![2]);

    app.apply(TuiAction::CycleProtocolFilter);
    assert_eq!(app.config_list.protocol_filter.as_deref(), Some("vless"));
    let visible: Vec<i64> = app.visible_configs().iter().map(|r| r.id).collect();
    assert_eq!(visible.len(), 2);
    assert!(visible.contains(&1));
    assert!(visible.contains(&3));

    app.apply(TuiAction::CycleProtocolFilter);
    assert!(app.config_list.protocol_filter.is_none());
    assert_eq!(app.visible_configs().len(), 3);
}

#[test]
fn cycles_config_filter() {
    use crate::tui::app::ConfigFilter;

    let enabled = row(1); // is_enabled=true, real_delay_ms=None after override
    let mut enabled = enabled;
    enabled.real_delay_ms = None;

    let mut disabled = row(2);
    disabled.is_enabled = false;
    disabled.real_delay_ms = None;

    let mut failed = row(3);
    failed.failure_reason = Some("timeout".to_string());
    failed.real_delay_ms = None;

    let has_delay = row(4); // real_delay_ms = Some(100) from default helper

    let data = TuiData::from_configs(vec![
        enabled.clone(),
        disabled.clone(),
        failed.clone(),
        has_delay.clone(),
    ]);
    let mut app = TuiApp::with_data(data);

    assert_eq!(app.config_list.filter, ConfigFilter::None);
    assert_eq!(app.visible_configs().len(), 4);

    app.apply(TuiAction::CycleFilter);
    assert_eq!(app.config_list.filter, ConfigFilter::EnabledOnly);
    let visible_ids: Vec<i64> = app.visible_configs().iter().map(|r| r.id).collect();
    assert!(visible_ids.contains(&1));
    assert!(!visible_ids.contains(&2));
    assert_eq!(visible_ids.len(), 3); // 1, 3, 4 are enabled

    app.apply(TuiAction::CycleFilter);
    assert_eq!(app.config_list.filter, ConfigFilter::FailedOnly);
    let visible_ids: Vec<i64> = app.visible_configs().iter().map(|r| r.id).collect();
    assert_eq!(visible_ids, vec![3]);

    app.apply(TuiAction::CycleFilter);
    assert_eq!(app.config_list.filter, ConfigFilter::HasDelay);
    let visible_ids: Vec<i64> = app.visible_configs().iter().map(|r| r.id).collect();
    assert_eq!(visible_ids, vec![4]);

    app.apply(TuiAction::CycleFilter);
    assert_eq!(app.config_list.filter, ConfigFilter::None);
    assert_eq!(app.visible_configs().len(), 4);
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
