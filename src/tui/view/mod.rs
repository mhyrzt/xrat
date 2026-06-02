mod chrome;
mod configs;
mod diagnostics;
mod modals;
mod runtime;
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
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(area);

    chrome::render_status_bar(frame, shell[0], app);
    render_body(frame, shell[1], app);
    chrome::render_key_bar(frame, shell[2]);

    if app.show_help {
        modals::render_help(frame, modals::centered_rect(68, 54, area));
    }

    if app.confirm.is_some() {
        modals::render_confirm(frame, modals::centered_rect(62, 34, area), app);
    }

    if app.import_modal.is_some() {
        modals::render_import_modal(frame, modals::centered_rect(72, 40, area), app);
    }

    if app.qr_modal.is_some() {
        modals::render_qr_modal(frame, modals::centered_rect(60, 80, area), app);
    }
}

fn render_body(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(22), Constraint::Min(20)])
        .split(area);

    chrome::render_mode_rail(frame, columns[0], app.active_view);
    match app.active_view {
        TuiView::Configs => configs::render(frame, columns[1], app),
        TuiView::Sources => sources::render(frame, columns[1], app),
        TuiView::Runtime => runtime::render(frame, columns[1], app),
        TuiView::Tests => tests::render(frame, columns[1], app),
        TuiView::Diagnostics => diagnostics::render(frame, columns[1], app),
    }
}
