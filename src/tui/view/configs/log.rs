use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::Line;

use crate::tui::app::TuiApp;
use crate::tui::theme;
use crate::tui::view::shared::{PanelStyle, render_scroll_panel};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
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

    render_scroll_panel(
        frame,
        area,
        failure_lines,
        &app.panel_scroll.log,
        PanelStyle {
            title: " Logs ",
            focused,
            right_pad: 0,
            wrap_trim: false,
        },
    );
}
