use crate::tui::app::{TuiAction, TuiApp, TuiView};

#[test]
fn switches_active_view() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::SwitchView(TuiView::Diagnostics));

    assert_eq!(app.active_view, TuiView::Diagnostics);
    assert_eq!(app.status_message, "");
}

#[test]
fn back_closes_help() {
    let mut app = TuiApp::default();

    app.apply(TuiAction::ShowHelp);
    app.apply(TuiAction::Back);

    assert!(!app.show_help);
    assert_eq!(app.status_message, "ready");
}
