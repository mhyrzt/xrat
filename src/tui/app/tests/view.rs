use crate::tui::app::{TuiAction, TuiApp, TuiLogTab, TuiView};

#[test]
fn switches_active_view() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::NextTab);

    assert_eq!(app.active_view, TuiView::Sources);
    assert_eq!(app.status_message, "");
}

#[test]
fn switches_log_tabs_and_resets_log_scroll() {
    let mut app = TuiApp::default();
    app.panel_scroll.log.set(5);

    app.apply(TuiAction::NextLogTab);

    assert_eq!(app.active_log_tab, TuiLogTab::ProxyEngine);
    assert_eq!(app.panel_scroll.log.get(), 0);
    assert_eq!(app.status_message, "proxy engine");

    app.panel_scroll.log.set(3);
    app.apply(TuiAction::PrevLogTab);

    assert_eq!(app.active_log_tab, TuiLogTab::XratEvents);
    assert_eq!(app.panel_scroll.log.get(), 0);
    assert_eq!(app.status_message, "xrat events");
}

#[test]
fn back_closes_help() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::ShowHelp);
    app.apply(TuiAction::Back);

    assert!(!app.show_help);
    assert_eq!(app.status_message, "ready");
}
