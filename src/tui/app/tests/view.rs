use crate::tui::app::{TuiAction, TuiApp, TuiLogTab, TuiView};

#[test]
fn switches_active_view() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::NextTab);

    assert_eq!(app.active_view, TuiView::Sources);
}

#[test]
fn switches_log_tabs_and_resets_log_scroll() {
    let mut app = TuiApp::default();
    app.panel_scroll.log.set(5);

    app.apply(TuiAction::NextLogTab);

    assert_eq!(app.active_log_tab, TuiLogTab::ProxyEngine);
    assert_eq!(app.panel_scroll.log.get(), 0);

    app.panel_scroll.log.set(3);
    app.apply(TuiAction::PrevLogTab);

    assert_eq!(app.active_log_tab, TuiLogTab::XratEvents);
    assert_eq!(app.panel_scroll.log.get(), 0);
}

#[test]
fn cycles_through_all_log_tabs() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::NextLogTab);
    assert_eq!(app.active_log_tab, TuiLogTab::ProxyEngine);
    app.apply(TuiAction::NextLogTab);
    assert_eq!(app.active_log_tab, TuiLogTab::Api);
    app.apply(TuiAction::NextLogTab);
    assert_eq!(app.active_log_tab, TuiLogTab::Stats);
    app.apply(TuiAction::NextLogTab);
    assert_eq!(app.active_log_tab, TuiLogTab::XratEvents);
    app.apply(TuiAction::PrevLogTab);
    assert_eq!(app.active_log_tab, TuiLogTab::Stats);
}

#[test]
fn clear_log_view_hides_events_without_deleting_records() {
    use crate::tui::data::logs::TuiEventLogRow;

    let mut app = TuiApp::default();
    app.data.logs.events = vec![
        TuiEventLogRow {
            id: 1,
            time: "2026-06-11 10:00:00".to_string(),
            level: "info".to_string(),
            source: "runtime".to_string(),
            kind: "start".to_string(),
            message: "started".to_string(),
        },
        TuiEventLogRow {
            id: 2,
            time: "2026-06-11 10:00:01".to_string(),
            level: "info".to_string(),
            source: "runtime".to_string(),
            kind: "stop".to_string(),
            message: "stopped".to_string(),
        },
    ];
    app.active_log_tab = TuiLogTab::XratEvents;

    app.apply(TuiAction::ClearLogView);

    // Records are untouched; only the view watermark advances.
    assert_eq!(app.data.logs.events.len(), 2);
    assert_eq!(app.events_clear_before_id, 2);
    assert!(!app.event_visible(2));
    assert!(app.event_visible(3));
}

#[test]
fn request_clear_events_arms_confirm() {
    use crate::tui::app::ConfirmKind;

    let mut app = TuiApp::default();
    app.apply(TuiAction::RequestClearEvents);

    let confirm = app.confirm.as_ref().expect("confirm should be armed");
    assert_eq!(confirm.kind, ConfirmKind::ClearEvents);

    // Cancelling clears the armed confirm without touching the database.
    app.apply(TuiAction::Cancel);
    assert!(app.confirm.is_none());
}

#[test]
fn back_closes_help() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::ShowHelp);
    app.apply(TuiAction::Back);

    assert!(!app.show_help);
}
