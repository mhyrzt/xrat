use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

use crate::tui::app::{TuiApp, TuiLogTab};
use crate::tui::theme;
use crate::tui::view::shared::{PanelStyle, render_scroll_panel};

const EVENT_TIME_WIDTH: usize = 19;
const EVENT_LEVEL_WIDTH: usize = 5;
const EVENT_SOURCE_WIDTH: usize = 12;
const EVENT_KIND_WIDTH: usize = 32;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    let lines = match app.active_log_tab {
        TuiLogTab::XratEvents => event_lines(app),
        TuiLogTab::ProxyEngine => proxy_lines(app),
        TuiLogTab::Stats => stats_lines(app),
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
            "{:<EVENT_TIME_WIDTH$}  {:<EVENT_LEVEL_WIDTH$}  {:<EVENT_SOURCE_WIDTH$}  {:<EVENT_KIND_WIDTH$}  {}",
            "TIME", "LEVEL", "SOURCE", "KIND", "MESSAGE"
        ),
        theme::muted_style(),
    )];
    lines.extend(app.data.logs.events.iter().map(|event| {
        Line::from(vec![
            Span::styled(
                format!("{:<EVENT_TIME_WIDTH$}", compact_time(&event.time)),
                theme::muted_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<EVENT_LEVEL_WIDTH$}",
                    truncate(&event.level, EVENT_LEVEL_WIDTH)
                ),
                level_style(&event.level),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<EVENT_SOURCE_WIDTH$}",
                    truncate(&event.source, EVENT_SOURCE_WIDTH)
                ),
                theme::chrome_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<EVENT_KIND_WIDTH$}",
                    truncate(&event.kind, EVENT_KIND_WIDTH)
                ),
                theme::chrome_style(),
            ),
            Span::raw("  "),
            Span::styled(event.message.as_str(), theme::chrome_style()),
        ])
    }));
    lines
}

fn proxy_lines(app: &TuiApp) -> Vec<Line<'_>> {
    use crate::tui::data::ProxyStream;

    if app.data.logs.proxy.is_empty() {
        return vec![Line::styled(
            "No proxy engine logs found for the latest runtime session.",
            theme::muted_style(),
        )];
    }

    let mut lines = vec![Line::styled(
        format!(
            "{:<19}  {:<7}  {:<8}  {:<10}  {}",
            "TIME", "LEVEL", "FEED", "SOURCE", "MESSAGE"
        ),
        theme::muted_style(),
    )];
    lines.extend(app.data.logs.proxy.iter().rev().map(|row| {
        let feed_style = if row.stream == ProxyStream::Stderr {
            theme::warning_style()
        } else {
            theme::muted_style()
        };
        Line::from(vec![
            Span::styled(
                format!("{:<19}", row.time.as_deref().unwrap_or("")),
                theme::muted_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<7}", row.level.as_deref().unwrap_or("")),
                proxy_level_style(row.level.as_deref()),
            ),
            Span::raw("  "),
            Span::styled(format!("{:<8}", row.engine), feed_style),
            Span::raw("  "),
            Span::styled(
                format!("{:<10}", row.component.as_deref().unwrap_or("")),
                theme::muted_style(),
            ),
            Span::raw("  "),
            Span::styled(row.message.as_str(), theme::chrome_style()),
        ])
    }));
    lines
}

fn proxy_level_style(level: Option<&str>) -> ratatui::style::Style {
    match level.map(str::to_ascii_lowercase).as_deref() {
        Some("error") => theme::failure_style(),
        Some("warning") | Some("warn") => theme::warning_style(),
        Some(_) => theme::accent_style(),
        None => theme::muted_style(),
    }
}

fn stats_lines(_app: &TuiApp) -> Vec<Line<'static>> {
    vec![Line::styled(
        "Live traffic stats are not wired up yet.",
        theme::muted_style(),
    )]
}

fn log_title(tab: TuiLogTab) -> Line<'static> {
    let mut spans = vec![
        Span::raw(" "),
        Span::styled("2:", theme::accent_style().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
    ];
    for (index, (log_tab, label)) in [
        (TuiLogTab::XratEvents, "xrat events"),
        (TuiLogTab::ProxyEngine, "proxy engine"),
        (TuiLogTab::Stats, "stats"),
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            spans.push(Span::raw("  "));
        }
        if tab == log_tab {
            spans.push(Span::styled(
                format!("[{label}]"),
                theme::accent_style().add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(label.to_string(), theme::muted_style()));
        }
    }
    spans.push(Span::raw(" "));
    Line::from(spans)
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

fn truncate(value: &str, width: usize) -> String {
    if value.width() <= width {
        return value.to_string();
    }

    let mut result = String::new();
    let mut used = 0usize;
    for ch in value.chars() {
        let char_width = ch.to_string().width();
        if used + char_width > width.saturating_sub(1) {
            break;
        }
        result.push(ch);
        used += char_width;
    }
    result.push('…');
    result
}
