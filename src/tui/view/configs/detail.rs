use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use crate::tui::app::TuiApp;
use crate::tui::data::TuiConfigRow;
use crate::tui::theme;
use crate::tui::view::shared::{append_bottom_lines, detail_line};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp) {
    let config = app.focused_config();
    render_detail(frame, area, config);
}

fn render_detail(frame: &mut Frame<'_>, area: Rect, config: Option<&TuiConfigRow>) {
    let lines = match config {
        Some(config) => {
            let mut lines = vec![
                Line::styled(
                    format!("#{} {}", config.id, config.display_name()),
                    theme::accent_style().add_modifier(Modifier::BOLD),
                ),
                Line::raw(""),
                detail_line("Protocol", &config.protocol),
                detail_line("Endpoint", config.endpoint()),
                detail_line("Network", config.network_label()),
                detail_line("Real Delay", config.delay_label()),
                detail_line(
                    "TCP",
                    config
                        .tcp_ms
                        .map(|tcp| format!("{tcp}ms"))
                        .unwrap_or_else(|| "-".to_string()),
                ),
                detail_line(
                    "Source",
                    config
                        .source_id
                        .map(|source_id| format!("#{source_id}"))
                        .unwrap_or_else(|| "-".to_string()),
                ),
                detail_line("Enabled", yes_no(config.is_enabled)),
                detail_line("Selected", yes_no(config.is_selected)),
                detail_line("Active", yes_no(config.is_active)),
                detail_line("Deleted", yes_no(config.is_deleted)),
                Line::raw(""),
                detail_line("Failure", config.failure_reason.as_deref().unwrap_or("-")),
            ];
            append_bottom_lines(
                &mut lines,
                vec![
                    Line::styled("Actions", theme::muted_style()),
                    Line::raw("[Enter]start  [Space]select  [e]nable  [x]disable"),
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
