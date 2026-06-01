use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::widgets::{Block, Borders, Cell, Row, Table};

use crate::tui::app::TuiApp;
use crate::tui::data::TuiTestResultRow;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);

    render_table(frame, columns[0], &app.data.tests.recent_results);
    super::detail::render(frame, columns[1], app);
}

fn render_table(frame: &mut Frame<'_>, area: Rect, results: &[TuiTestResultRow]) {
    let header = Row::new(["ID", "Config", "Status", "Real", "TCP", "Tested"])
        .style(theme::accent_style().add_modifier(Modifier::BOLD));
    let rows = results.iter().map(|result| {
        let style = if result.failure_reason.is_some() {
            theme::failure_style()
        } else {
            theme::chrome_style()
        };
        Row::new(vec![
            Cell::from(result.id.to_string()),
            Cell::from(format!("#{}", result.config_id)),
            Cell::from(result.status.clone()),
            Cell::from(
                result
                    .real_delay_ms
                    .map(|delay| format!("{delay}ms"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            Cell::from(
                result
                    .tcp_ms
                    .map(|delay| format!("{delay}ms"))
                    .unwrap_or_else(|| "-".to_string()),
            ),
            Cell::from(result.tested_at.clone()),
        ])
        .style(style)
    });

    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(18),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(" Recent Results ")
            .borders(Borders::ALL),
    )
    .column_spacing(1);
    frame.render_widget(table, area);
}
