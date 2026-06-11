mod detail;
mod filter;
mod log;
mod runtime;
mod source_detail;
mod sources_table;
mod stats;
mod table;
mod testbar;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::tui::app::{TuiApp, TuiPanel, TuiView};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(6)])
        .split(area);

    let chrome = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(sections[0]);

    filter::render(frame, chrome[0], app);
    testbar::render(frame, chrome[1], app);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(sections[1]);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(rows[0]);

    let panel = app.focused_panel;
    match app.active_view {
        TuiView::Configs => {
            table::render(frame, top[0], app, panel == TuiPanel::Table);
            detail::render(frame, top[1], app, panel == TuiPanel::Detail);
        }
        TuiView::Sources => {
            sources_table::render(frame, top[0], app, panel == TuiPanel::Table);
            source_detail::render(frame, top[1], app, panel == TuiPanel::Detail);
        }
    }

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(rows[1]);

    log::render(frame, bottom[0], app, panel == TuiPanel::Log);
    runtime::render(frame, bottom[1], app, panel == TuiPanel::Runtime);
}
