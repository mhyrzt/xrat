use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::data::TuiConfigRow;
use crate::tui::theme;
use crate::tui::view::shared::{append_bottom_lines, push_detail};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let config = app.focused_config();
    render_detail(frame, area, config);
}

const LABEL_WIDTH: usize = 12;

fn render_detail(frame: &mut Frame<'_>, area: Rect, config: Option<&TuiConfigRow>) {
    let content_width = area.width.saturating_sub(2) as usize;
    let lines = match config {
        Some(config) => {
            let mut lines = vec![
                Line::styled(
                    format!("#{} {}", config.id, config.display_name()),
                    theme::accent_style().add_modifier(Modifier::BOLD),
                ),
                Line::raw(""),
            ];
            push_detail(
                &mut lines,
                "Protocol",
                &config.protocol,
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Endpoint",
                config.endpoint(),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Network",
                config.network_label(),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Real Delay",
                config.delay_label(),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "TCP",
                config
                    .tcp_ms
                    .map(|tcp| format!("{tcp}ms"))
                    .unwrap_or_else(|| "-".to_string()),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Source",
                config
                    .source_id
                    .map(|source_id| format!("#{source_id}"))
                    .unwrap_or_else(|| "-".to_string()),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Enabled",
                yes_no(config.is_enabled),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Active",
                yes_no(config.is_active),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(
                &mut lines,
                "Deleted",
                yes_no(config.is_deleted),
                LABEL_WIDTH,
                content_width,
            );
            lines.push(Line::raw(""));
            push_detail(
                &mut lines,
                "Failure",
                config.failure_reason.as_deref().unwrap_or("-"),
                LABEL_WIDTH,
                content_width,
            );
            append_bottom_lines(
                &mut lines,
                vec![
                    Line::styled("Actions", theme::muted_style()),
                    Line::raw("[Enter]start  [e]nable  [x]disable"),
                    Line::raw("[t]est  [a]ll  [v]isible  [d]elete  [r]estore"),
                ],
                area,
                1,
            );
            lines
        }
        None => vec![
            Line::styled(
                "No configs",
                theme::accent_style().add_modifier(Modifier::BOLD),
            ),
            Line::raw(""),
            Line::raw("Import configs with `xrat import <input>`"),
            Line::raw("or add one with `xrat add <config-uri>`."),
        ],
    };

    use ratatui::widgets::BorderType;
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Detail ")
                    .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
                    .border_type(BorderType::Plain),
            )
            .style(theme::chrome_style())
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
