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
fn selects_log_tab_directly_and_resets_scroll() {
    let mut app = TuiApp::default();
    app.panel_scroll.log.set(7);

    app.apply(TuiAction::SelectLogTab(TuiLogTab::Stats));

    assert_eq!(app.active_log_tab, TuiLogTab::Stats);
    assert_eq!(app.panel_scroll.log.get(), 0);
}

#[test]
fn cycles_through_all_three_log_tabs() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::NextLogTab);
    assert_eq!(app.active_log_tab, TuiLogTab::ProxyEngine);
    app.apply(TuiAction::NextLogTab);
    assert_eq!(app.active_log_tab, TuiLogTab::Stats);
    app.apply(TuiAction::NextLogTab);
    assert_eq!(app.active_log_tab, TuiLogTab::XratEvents);
    app.apply(TuiAction::PrevLogTab);
    assert_eq!(app.active_log_tab, TuiLogTab::Stats);
}

#[test]
fn back_closes_help() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::ShowHelp);
    app.apply(TuiAction::Back);

    assert!(!app.show_help);
}
