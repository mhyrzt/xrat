use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render_help(frame: &mut Frame<'_>, area: Rect) {
    frame.render_widget(Clear, area);
    let text = vec![
        Line::styled("XRAT TUI Help", theme::accent_style().bold()),
        Line::raw(""),
        Line::raw("1-4       switch views"),
        Line::raw("j/k       move focus"),
        Line::raw("arrows    move focus"),
        Line::raw("/         edit config search"),
        Line::raw("f         show/hide deleted configs"),
        Line::raw("s         cycle config sort (configs) / start batch (tests)"),
        Line::raw("c         cancel running test batch (tests view)"),
        Line::raw("Space     select focused config"),
        Line::raw("e/x       enable/disable focused config"),
        Line::raw("d/D       soft delete / purge focused config"),
        Line::raw("r         restore focused deleted config"),
        Line::raw("Ctrl+U    clear search while editing"),
        Line::raw("Esc       close modal/back"),
        Line::raw("q/Ctrl+C  quit"),
    ];
    let block = Block::default().title(" Help ").borders(Borders::ALL);
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Left)
            .style(theme::chrome_style()),
        area,
    );
}

pub fn render_confirm(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let Some(confirm) = &app.confirm else {
        return;
    };

    frame.render_widget(Clear, area);
    let text = vec![
        Line::styled(&confirm.message, theme::chrome_style()),
        Line::raw(""),
        Line::styled("Enter/y confirm   Esc/n cancel", theme::muted_style()),
    ];
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(confirm.title.as_str())
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);
    horizontal[1]
}
