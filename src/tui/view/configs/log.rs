use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let mut failure_lines: Vec<Line<'_>> = app
        .data
        .configs
        .iter()
        .filter_map(|config| {
            config.failure_reason.as_deref().map(|reason| {
                Line::styled(
                    format!("FAIL config #{}: {reason}", config.id),
                    theme::failure_style(),
                )
            })
        })
        .take(8)
        .collect();

    if !failure_lines.is_empty() && !app.event_log.is_empty() {
        failure_lines.push(Line::raw(""));
    }

    let event_lines: Vec<Line<'_>> = if app.event_log.is_empty() {
        vec![Line::styled(
            "No events recorded yet.",
            theme::muted_style(),
        )]
    } else {
        app.event_log
            .iter()
            .rev()
            .map(|entry| {
                let style = if entry.starts_with("ERR") {
                    theme::failure_style()
                } else {
                    theme::chrome_style()
                };
                Line::styled(entry.as_str(), style)
            })
            .collect()
    };
    failure_lines.extend(event_lines);

    frame.render_widget(
        Paragraph::new(failure_lines)
            .block(
                Block::default()
                    .title(" Failures and Event Log ")
                    .borders(Borders::ALL),
            )
            .style(theme::chrome_style())
            .wrap(Wrap { trim: false }),
        area,
    );
}
