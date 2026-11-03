use ratatui::layout::{Constraint, Direction, Layout, Position, Rect};

use crate::tui::app::TuiPanel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashboardLayout {
    pub screen: Rect,
    pub body: Rect,
    pub keybar: Rect,
    pub filter: Rect,
    pub testbar: Rect,
    pub table: Rect,
    pub detail: Rect,
    pub log: Rect,
    pub runtime: Rect,
}

impl DashboardLayout {
    pub fn new(screen: Rect) -> Self {
        let shell = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(1)])
            .split(screen);

        let body = shell[0];
        let keybar = shell[1];

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(6)])
            .split(body);

        let chrome = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(sections[0]);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
            .split(sections[1]);

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(rows[0]);

        let bottom = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
            .split(rows[1]);

        Self {
            screen,
            body,
            keybar,
            filter: chrome[0],
            testbar: chrome[1],
            table: top[0],
            detail: top[1],
            log: bottom[0],
            runtime: bottom[1],
        }
    }

    pub fn panel_at(&self, col: u16, row: u16) -> Option<TuiPanel> {
        let pos = Position::new(col, row);
        if self.table.contains(pos) {
            Some(TuiPanel::Table)
        } else if self.detail.contains(pos) {
            Some(TuiPanel::Detail)
        } else if self.log.contains(pos) {
            Some(TuiPanel::Log)
        } else if self.runtime.contains(pos) {
            Some(TuiPanel::Runtime)
        } else {
            None
        }
    }
}
