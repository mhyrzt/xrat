mod chrome;
mod configs;
mod diagnostics;
mod modals;
mod shared;
mod sources;
mod tests;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::tui::app::{TuiApp, TuiView};

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(area);

    render_body(frame, shell[0], app);
    chrome::render_key_bar(frame, shell[1], app.active_view);

    if app.show_help {
        modals::render_help(frame, modals::centered_rect(82, 42, area));
    }

    if app.confirm.is_some() {
        modals::render_confirm(frame, modals::centered_rect(62, 34, area), app);
    }

    if app.import_modal.is_some() {
        modals::render_import_modal(frame, modals::centered_rect(72, 40, area), app);
    }

    if app.rename_modal.is_some() {
        modals::render_rename_modal(frame, modals::centered_rect(60, 30, area), app);
    }

    if app.qr_modal.is_some() {
        modals::render_qr_modal(frame, modals::centered_rect(60, 80, area), app);
    }
}

fn render_body(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    match app.active_view {
        TuiView::Configs => configs::render(frame, area, app),
        TuiView::Sources => sources::render(frame, area, app),
        TuiView::Tests => tests::render(frame, area, app),
        TuiView::Diagnostics => diagnostics::render(frame, area, app),
    }
}
