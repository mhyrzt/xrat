mod chrome;
mod configs;
mod modals;
mod shared;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::tui::app::TuiApp;

pub fn render(frame: &mut Frame<'_>, app: &TuiApp) {
    let area = frame.area();
    let shell = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(1)])
        .split(area);

    render_body(frame, shell[0], app);
    chrome::render_key_bar(frame, shell[1]);

    if app.show_help {
        modals::render_help(frame, area);
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
    configs::render(frame, area, app);
}
