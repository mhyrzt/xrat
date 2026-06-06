use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};

use crate::tui::app::{TuiApp, TuiLogTab};
use crate::tui::theme;
use crate::tui::view::shared::{PanelStyle, render_scroll_panel};

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    let lines = match app.active_log_tab {
        TuiLogTab::XratEvents => event_lines(app),
        TuiLogTab::ProxyEngine => proxy_lines(app),
    };

    render_scroll_panel(
        frame,
        area,
        lines,
        &app.panel_scroll.log,
        PanelStyle {
            title: log_title(app.active_log_tab),
            focused,
            right_pad: 0,
            wrap_trim: false,
        },
    );
}

fn event_lines(app: &TuiApp) -> Vec<Line<'_>> {
    if app.data.logs.events.is_empty() {
        return vec![Line::styled(
            "No xrat events recorded yet.",
            theme::muted_style(),
        )];
    }

    let mut lines = vec![Line::styled(
        format!(
            "{:<19}  {:<5}  {:<12}  {:<14}  {}",
            "TIME", "LEVEL", "SOURCE", "KIND", "MESSAGE"
        ),
        theme::muted_style(),
    )];
    lines.extend(app.data.logs.events.iter().map(|event| {
        Line::from(vec![
            Span::styled(
                format!("{:<19}", compact_time(&event.time)),
                theme::muted_style(),
            ),
            Span::raw("  "),
            Span::styled(format!("{:<5}", event.level), level_style(&event.level)),
            Span::raw("  "),
            Span::styled(format!("{:<12}", event.source), theme::chrome_style()),
            Span::raw("  "),
            Span::styled(format!("{:<14}", event.kind), theme::chrome_style()),
            Span::raw("  "),
            Span::styled(event.message.as_str(), theme::chrome_style()),
        ])
    }));
    lines
}

fn proxy_lines(app: &TuiApp) -> Vec<Line<'_>> {
    if app.data.logs.proxy.is_empty() {
        return vec![Line::styled(
            "No proxy engine logs found for the latest runtime session.",
            theme::muted_style(),
        )];
    }

    let mut lines = vec![Line::styled(
        format!("{:<12}  {}", "FEED", "MESSAGE"),
        theme::muted_style(),
    )];
    lines.extend(app.data.logs.proxy.iter().rev().map(|row| {
        let feed_style = if row.feed.ends_with("err") {
            theme::warning_style()
        } else {
            theme::muted_style()
        };
        Line::from(vec![
            Span::styled(format!("{:<12}", row.feed), feed_style),
            Span::raw("  "),
            Span::styled(row.line.as_str(), theme::chrome_style()),
        ])
    }));
    lines
}

fn log_title(tab: TuiLogTab) -> &'static str {
    match tab {
        TuiLogTab::XratEvents => " Logs  [xrat events]  proxy engine  ",
        TuiLogTab::ProxyEngine => " Logs   xrat events  [proxy engine] ",
    }
}

fn compact_time(value: &str) -> String {
    value
        .trim_end_matches('Z')
        .replace('T', " ")
        .chars()
        .take(19)
        .collect()
}

fn level_style(level: &str) -> ratatui::style::Style {
    match level {
        "error" => theme::failure_style(),
        "warn" => theme::warning_style(),
        _ => theme::accent_style(),
    }
}
