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
const COLUMN_GAP: usize = 2;

const EVENT_PREFIX_WIDTH: usize = EVENT_TIME_WIDTH
    + COLUMN_GAP
    + EVENT_LEVEL_WIDTH
    + COLUMN_GAP
    + EVENT_SOURCE_WIDTH
    + COLUMN_GAP
    + EVENT_KIND_WIDTH
    + COLUMN_GAP;

const PROXY_TIME_WIDTH: usize = 19;
const PROXY_LEVEL_WIDTH: usize = 7;
const PROXY_FEED_WIDTH: usize = 8;
const PROXY_SOURCE_WIDTH: usize = 10;
const PROXY_PREFIX_WIDTH: usize = PROXY_TIME_WIDTH
    + COLUMN_GAP
    + PROXY_LEVEL_WIDTH
    + COLUMN_GAP
    + PROXY_FEED_WIDTH
    + COLUMN_GAP
    + PROXY_SOURCE_WIDTH
    + COLUMN_GAP;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    let content_width = area.width.saturating_sub(2) as usize;
    let lines = match app.active_log_tab {
        TuiLogTab::XratEvents => event_lines(app, content_width),
        TuiLogTab::ProxyEngine => proxy_lines(app, content_width),
        TuiLogTab::Stats => stats_lines(app),
    };

    let viewport = render_scroll_panel(
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
    app.panel_viewport.log.set(viewport);
}

fn event_lines(app: &TuiApp, content_width: usize) -> Vec<Line<'_>> {
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
    for event in &app.data.logs.events {
        let prefix = vec![
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
        ];
        push_wrapped_row(
            &mut lines,
            prefix,
            EVENT_PREFIX_WIDTH,
            &event.message,
            content_width,
        );
    }
    lines
}

fn proxy_lines(app: &TuiApp, content_width: usize) -> Vec<Line<'_>> {
    use crate::tui::data::ProxyStream;

    if app.data.logs.proxy.is_empty() {
        return vec![Line::styled(
            "No proxy engine logs found for the latest runtime session.",
            theme::muted_style(),
        )];
    }

    let mut lines = vec![Line::styled(
        format!(
            "{:<PROXY_TIME_WIDTH$}  {:<PROXY_LEVEL_WIDTH$}  {:<PROXY_FEED_WIDTH$}  {:<PROXY_SOURCE_WIDTH$}  {}",
            "TIME", "LEVEL", "FEED", "SOURCE", "MESSAGE"
        ),
        theme::muted_style(),
    )];
    for row in app.data.logs.proxy.iter().rev() {
        let feed_style = if row.stream == ProxyStream::Stderr {
            theme::warning_style()
        } else {
            theme::muted_style()
        };
        let prefix = vec![
            Span::styled(
                format!("{:<PROXY_TIME_WIDTH$}", row.time.as_deref().unwrap_or("")),
                theme::muted_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<PROXY_LEVEL_WIDTH$}", row.level.as_deref().unwrap_or("")),
                proxy_level_style(row.level.as_deref()),
            ),
            Span::raw("  "),
            Span::styled(format!("{:<PROXY_FEED_WIDTH$}", row.engine), feed_style),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<PROXY_SOURCE_WIDTH$}",
                    row.component.as_deref().unwrap_or("")
                ),
                theme::muted_style(),
            ),
            Span::raw("  "),
        ];
        push_wrapped_row(
            &mut lines,
            prefix,
            PROXY_PREFIX_WIDTH,
            &row.message,
            content_width,
        );
    }
    lines
}

fn push_wrapped_row<'a>(
    lines: &mut Vec<Line<'a>>,
    prefix: Vec<Span<'a>>,
    prefix_width: usize,
    message: &str,
    content_width: usize,
) {
    let avail = content_width.saturating_sub(prefix_width).max(1);
    let chunks = wrap_message(message, avail);

    let mut first = prefix;
    first.push(Span::styled(chunks[0].clone(), theme::chrome_style()));
    lines.push(Line::from(first));

    for chunk in &chunks[1..] {
        lines.push(Line::from(vec![
            Span::raw(" ".repeat(prefix_width)),
            Span::styled(chunk.clone(), theme::chrome_style()),
        ]));
    }
}

fn wrap_message(message: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![message.to_string()];
    }

    let mut tokens: Vec<String> = Vec::new();
    for word in message.split_whitespace() {
        if word.width() > width {
            tokens.extend(hard_break(word, width));
        } else {
            tokens.push(word.to_string());
        }
    }

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;
    for token in tokens {
        let token_width = token.width();
        let separator = usize::from(!current.is_empty());
        if !current.is_empty() && current_width + separator + token_width > width {
            lines.push(std::mem::take(&mut current));
            current_width = 0;
        }
        if !current.is_empty() {
            current.push(' ');
            current_width += 1;
        }
        current.push_str(&token);
        current_width += token_width;
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

fn hard_break(word: &str, width: usize) -> Vec<String> {
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_width = 0usize;
    for ch in word.chars() {
        let char_width = ch.to_string().width();
        if current_width + char_width > width && !current.is_empty() {
            chunks.push(std::mem::take(&mut current));
            current_width = 0;
        }
        current.push(ch);
        current_width += char_width;
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
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

#[cfg(test)]
mod tests {
    use super::{hard_break, wrap_message};
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn wraps_message_on_word_boundaries_within_width() {
        let chunks = wrap_message("Reconnected config 3 after stale runtime PID", 20);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.width() <= 20, "chunk too wide: {chunk:?}");
        }
        assert_eq!(
            chunks.join(" "),
            "Reconnected config 3 after stale runtime PID"
        );
    }

    #[test]
    fn hard_breaks_words_longer_than_width() {
        let chunks = hard_break("daemon_restart_stale_pid_recovered", 10);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.width() <= 10);
        }
        assert_eq!(chunks.concat(), "daemon_restart_stale_pid_recovered");
    }

    #[test]
    fn wrap_returns_single_chunk_for_short_message() {
        assert_eq!(wrap_message("ok", 40), vec!["ok".to_string()]);
    }

    #[test]
    fn wrap_zero_width_returns_message_unchanged() {
        assert_eq!(wrap_message("anything", 0), vec!["anything".to_string()]);
    }
}
