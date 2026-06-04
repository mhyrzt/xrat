mod detail;
mod filter;
mod log;
mod runtime;
mod table;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::tui::app::TuiApp;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(6)])
        .split(area);

    filter::render(frame, sections[0], app);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(sections[1]);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(rows[0]);

    table::render(frame, top[0], app);
    detail::render(frame, top[1], app);

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(rows[1]);

    log::render(frame, bottom[0], app);
    runtime::render(frame, bottom[1], app);
}
