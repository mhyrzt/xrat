use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::theme;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(4)])
        .split(area);

    render_info(frame, sections[0], app);
    render_log(frame, sections[1], app);
}

fn render_info(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let task_label = app.task_state.label();
    let runtime_state = app.data.runtime.status.clone();
    let active_config = app
        .data
        .runtime
        .active_config_id
        .map(|id| format!("config #{id}"))
        .unwrap_or_else(|| "none".to_string());

    let lines = vec![
        Line::from(vec![
            Span::styled("Runtime:       ", theme::muted_style()),
            Span::raw(&runtime_state),
        ]),
        Line::from(vec![
            Span::styled("Active config: ", theme::muted_style()),
            Span::raw(active_config),
        ]),
        Line::from(vec![
            Span::styled("Task:          ", theme::muted_style()),
            Span::raw(task_label),
        ]),
        Line::from(vec![
            Span::styled("Configs:       ", theme::muted_style()),
            Span::raw(format!(
                "{} total  {} enabled  {} selected  {} deleted  {} failed",
                app.data.total_configs,
                app.data.enabled_configs,
                app.data.selected_configs,
                app.data.deleted_configs,
                app.data.failed_configs,
            )),
        ]),
        Line::from(vec![
            Span::styled("Sources:       ", theme::muted_style()),
            Span::raw(app.data.sources.len().to_string()),
        ]),
        Line::from(vec![
            Span::styled("Database:      ", theme::muted_style()),
            Span::raw(app.data.db_label.as_str()),
        ]),
        Line::from(vec![
            Span::styled("Config file:   ", theme::muted_style()),
            Span::raw(app.data.config_path.as_str()),
        ]),
        Line::from(vec![
            Span::styled("Log entries:   ", theme::muted_style()),
            Span::raw(app.event_log.len().to_string()),
        ]),
    ];

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().title(" Info ").borders(Borders::ALL))
            .style(theme::chrome_style()),
        area,
    );
}

fn render_log(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let lines: Vec<Line<'_>> = if app.event_log.is_empty() {
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

    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Event Log (newest first) ")
                    .borders(Borders::ALL),
            )
            .style(theme::chrome_style())
            .wrap(Wrap { trim: false }),
        area,
    );
}
