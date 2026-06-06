use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::Line;

use crate::tui::app::TuiApp;
use crate::tui::data::TuiConfigRow;
use crate::tui::theme;
use crate::tui::view::shared::{PanelStyle, push_detail, render_scroll_panel};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    let config = app.focused_config();
    let source_label = config
        .map(|config| source_label(app, config))
        .unwrap_or_default();
    let lines = detail_lines(area, config, &source_label);
    render_scroll_panel(
        frame,
        area,
        lines,
        &app.panel_scroll.detail,
        PanelStyle {
            title: " Detail ",
            focused,
            right_pad: RIGHT_PAD,
            wrap_trim: true,
        },
    );
}

const LABEL_WIDTH: usize = crate::tui::theme::DETAIL_LABEL_WIDTH;
const RIGHT_PAD: u16 = crate::tui::theme::DETAIL_RIGHT_PAD;

fn source_label(app: &TuiApp, config: &TuiConfigRow) -> String {
    match config.source_id {
        Some(source_id) => app
            .data
            .sources
            .iter()
            .find(|source| source.id == source_id)
            .map(|source| format!("{} {}", source.display_ref(), source.display_name()))
            .unwrap_or_else(|| format!("#{source_id}")),
        None => "none".to_string(),
    }
}

fn detail_lines<'a>(
    area: Rect,
    config: Option<&'a TuiConfigRow>,
    source_label: &'a str,
) -> Vec<Line<'a>> {
    let content_width = area.width.saturating_sub(2 + RIGHT_PAD) as usize;
    match config {
        Some(config) => {
            let mut lines = vec![
                Line::styled(
                    format!("{} {}", config.display_ref(), config.display_name()),
                    theme::accent_style().add_modifier(Modifier::BOLD),
                ),
                Line::raw(""),
            ];
            push_detail(
                &mut lines,
                "ID",
                config.id.to_string(),
                LABEL_WIDTH,
                content_width,
            );
            push_detail(&mut lines, "Ref", &config.r#ref, LABEL_WIDTH, content_width);
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
                source_label,
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
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}
