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

const API_METHOD_WIDTH: usize = 7;
const API_PATH_WIDTH: usize = 24;
const API_CODE_WIDTH: usize = 4;
const API_PREFIX_WIDTH: usize = EVENT_TIME_WIDTH
    + COLUMN_GAP
    + EVENT_LEVEL_WIDTH
    + COLUMN_GAP
    + API_METHOD_WIDTH
    + COLUMN_GAP
    + API_PATH_WIDTH
    + COLUMN_GAP
    + API_CODE_WIDTH
    + COLUMN_GAP;

const PROXY_TIME_WIDTH: usize = 19;
const PROXY_LEVEL_WIDTH: usize = 7;
const PROXY_SOURCE_WIDTH: usize = 16;
const PROXY_PREFIX_WIDTH: usize = PROXY_TIME_WIDTH
    + COLUMN_GAP
    + PROXY_LEVEL_WIDTH
    + COLUMN_GAP
    + PROXY_SOURCE_WIDTH
    + COLUMN_GAP;

pub fn render(frame: &mut Frame<'_>, area: Rect, app: &TuiApp, focused: bool) {
    if app.active_log_tab == TuiLogTab::Stats {
        super::stats::render(
            frame,
            area,
            app,
            log_title(TuiLogTab::Stats, active_engine_label(app)),
            focused,
        );
        app.panel_viewport.log.set(area.height.saturating_sub(2));
        return;
    }

    let content_width = area.width.saturating_sub(2) as usize;
    let lines = match app.active_log_tab {
        TuiLogTab::XratEvents => event_lines(app, content_width),
        TuiLogTab::ProxyEngine => proxy_lines(app, content_width),
        TuiLogTab::Api => api_lines(app, content_width),
        TuiLogTab::Stats => unreachable!("stats tab handled above"),
    };

    let viewport = render_scroll_panel(
        frame,
        area,
        lines,
        &app.panel_scroll.log,
        PanelStyle {
            title: log_title(app.active_log_tab, active_engine_label(app)),
            focused,
            right_pad: 0,
            wrap_trim: false,
            header: log_header(app.active_log_tab),
        },
    );
    app.panel_viewport.log.set(viewport);
}

fn event_lines(app: &TuiApp, content_width: usize) -> Vec<Line<'_>> {
    let events: Vec<_> = app
        .data
        .logs
        .events
        .iter()
        .filter(|event| app.event_visible(event.id))
        .collect();
    if events.is_empty() {
        return vec![Line::styled(
            "No xrat events recorded yet.",
            theme::muted_style(),
        )];
    }

    let mut lines = Vec::new();
    for event in events {
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
                theme::severity_style(&event.level),
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

fn api_lines(app: &TuiApp, content_width: usize) -> Vec<Line<'_>> {
    let events: Vec<_> = app
        .data
        .logs
        .events
        .iter()
        .filter(|event| event.source == "api" && app.event_visible(event.id))
        .collect();
    if events.is_empty() {
        return vec![Line::styled(
            "No API requests recorded yet.",
            theme::muted_style(),
        )];
    }

    let mut lines = Vec::new();
    for event in events {
        let (method, path, code, message) = parse_api_request(&event.kind, &event.message);
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
                theme::severity_style(&event.level),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<API_METHOD_WIDTH$}", truncate(&method, API_METHOD_WIDTH)),
                theme::chrome_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<API_PATH_WIDTH$}", truncate(&path, API_PATH_WIDTH)),
                theme::chrome_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<API_CODE_WIDTH$}", truncate(&code, API_CODE_WIDTH)),
                theme::severity_style(&event.level),
            ),
            Span::raw("  "),
        ];
        push_wrapped_row(
            &mut lines,
            prefix,
            API_PREFIX_WIDTH,
            &message,
            content_width,
        );
    }
    lines
}

/// Split a recorded API event into table columns. The server middleware records
/// each request as the synthetic line `"{method} {path} -> {code}"`, which is not
/// a real axum/handler message, so its MESSAGE column shows `-`. Anything that
/// does not match that shape is treated as an actual message and shown verbatim.
fn parse_api_request(kind: &str, message: &str) -> (String, String, String, String) {
    if let Some((request, code)) = message.rsplit_once(" -> ")
        && code.chars().all(|ch| ch.is_ascii_digit())
        && let Some((method, path)) = request.split_once(' ')
    {
        return (
            method.to_string(),
            path.to_string(),
            code.to_string(),
            "-".to_string(),
        );
    }
    (
        kind.to_string(),
        "-".to_string(),
        "-".to_string(),
        message.to_string(),
    )
}

fn proxy_lines(app: &TuiApp, content_width: usize) -> Vec<Line<'_>> {
    use crate::tui::data::ProxyStream;

    let visible: Vec<_> = visible_proxy_rows(app)
        .iter()
        .filter(|row| !row.is_self_api_traffic())
        .collect();
    if visible.is_empty() {
        return vec![Line::styled(
            "No proxy engine logs found for the latest runtime session.",
            theme::muted_style(),
        )];
    }

    let mut lines = Vec::new();
    for row in visible.iter().rev() {
        let source_style = if row.stream == ProxyStream::Stderr {
            theme::warning_style()
        } else {
            theme::muted_style()
        };
        let prefix = vec![
            Span::styled(
                format!(
                    "{:<PROXY_TIME_WIDTH$}",
                    row.time.as_deref().map(compact_time).unwrap_or_default()
                ),
                theme::muted_style(),
            ),
            Span::raw("  "),
            Span::styled(
                format!("{:<PROXY_LEVEL_WIDTH$}", row.level.as_deref().unwrap_or("")),
                proxy_severity_style(row),
            ),
            Span::raw("  "),
            Span::styled(
                format!(
                    "{:<PROXY_SOURCE_WIDTH$}",
                    truncate(row.component.as_deref().unwrap_or(""), PROXY_SOURCE_WIDTH)
                ),
                source_style,
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

/// Proxy rows after the view-only clear boundary. With a clear signature set,
/// rows up to and including its most recent occurrence stay hidden; a rotated
/// log whose signature is gone shows everything again.
fn visible_proxy_rows(app: &TuiApp) -> &[crate::tui::data::TuiProxyLogRow] {
    let rows = &app.data.logs.proxy;
    let Some(signature) = &app.proxy_clear_signature else {
        return rows;
    };
    match rows.iter().rposition(|row| &row.signature() == signature) {
        Some(index) => &rows[index + 1..],
        None => rows,
    }
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

/// Severity color for a proxy engine row. Uses the parsed level when present;
/// otherwise infers severity from message keywords and stderr context so
/// unparseable lines still get a sensible color without hiding the raw text.
fn proxy_severity_style(row: &crate::tui::data::TuiProxyLogRow) -> ratatui::style::Style {
    use crate::tui::data::ProxyStream;

    if let Some(level) = row.level.as_deref().filter(|level| !level.is_empty()) {
        return theme::severity_style(level);
    }

    let message = row.message.to_ascii_lowercase();
    if message.contains("panic") || message.contains("fatal") || message.contains("error") {
        theme::failure_style()
    } else if message.contains("warn") || row.stream == ProxyStream::Stderr {
        theme::warning_style()
    } else {
        theme::muted_style()
    }
}

/// Column header pinned above the scrolling body for the tabular tabs. The
/// stats tab has no columns, so it returns `None`.
fn log_header(tab: TuiLogTab) -> Option<Line<'static>> {
    let text = match tab {
        TuiLogTab::XratEvents => format!(
            "{:<EVENT_TIME_WIDTH$}  {:<EVENT_LEVEL_WIDTH$}  {:<EVENT_SOURCE_WIDTH$}  {:<EVENT_KIND_WIDTH$}  {}",
            "TIME", "LEVEL", "SOURCE", "KIND", "MESSAGE"
        ),
        TuiLogTab::ProxyEngine => format!(
            "{:<PROXY_TIME_WIDTH$}  {:<PROXY_LEVEL_WIDTH$}  {:<PROXY_SOURCE_WIDTH$}  {}",
            "TIME", "LEVEL", "SOURCE", "MESSAGE"
        ),
        TuiLogTab::Api => format!(
            "{:<EVENT_TIME_WIDTH$}  {:<EVENT_LEVEL_WIDTH$}  {:<API_METHOD_WIDTH$}  {:<API_PATH_WIDTH$}  {:<API_CODE_WIDTH$}  {}",
            "TIME", "LEVEL", "METHOD", "PATH", "CODE", "MESSAGE"
        ),
        TuiLogTab::Stats => return None,
    };
    Some(Line::styled(text, theme::muted_style()))
}

fn log_title(tab: TuiLogTab, engine_label: Option<String>) -> Line<'static> {
    let mut spans = vec![
        Span::raw(" "),
        Span::styled("2:", theme::accent_style().add_modifier(Modifier::BOLD)),
        Span::raw("  "),
    ];
    for (index, (log_tab, label)) in [
        (TuiLogTab::XratEvents, "Events"),
        (TuiLogTab::ProxyEngine, "Engine"),
        (TuiLogTab::Stats, "Traffic"),
        (TuiLogTab::Api, "API"),
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
    if matches!(tab, TuiLogTab::ProxyEngine | TuiLogTab::Stats)
        && let Some(engine_label) = engine_label
    {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("· {engine_label}"),
            theme::accent_style(),
        ));
    }
    spans.push(Span::raw(" "));
    Line::from(spans)
}

/// Engine name (and version, if probed) currently writing to the engine tab,
/// taken from the most recent proxy row. `None` when no engine logs are loaded.
fn active_engine_label(app: &TuiApp) -> Option<String> {
    let engine = app.data.logs.proxy.last().map(|row| row.engine.clone())?;
    let version = app
        .engines
        .iter()
        .find(|info| info.name == engine)
        .and_then(|info| info.version.clone());
    Some(match version {
        Some(version) => format!("{engine} {version}"),
        None => engine,
    })
}

/// Normalize a timestamp to `YYYY-MM-DD HH:MM:SS` (19 chars): drop a trailing
/// `Z`, swap the ISO `T`, map xray's `/` date separators to `-`, and trim any
/// sub-second fraction. Shared by every log tab so columns line up identically.
fn compact_time(value: &str) -> String {
    value
        .trim_end_matches('Z')
        .replace('T', " ")
        .replace('/', "-")
        .chars()
        .take(19)
        .collect()
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
