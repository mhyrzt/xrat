mod detail;
mod progress;
mod results;

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::tui::app::TuiApp;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8)])
        .split(area);

    progress::render(frame, sections[0], app);
    results::render(frame, sections[1], app);
}
