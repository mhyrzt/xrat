//! Proxy engine (xray / sing-box) log parsing for the TUI logs card. Raw
//! process stdout/stderr lines are parsed into structured fields where the
//! format is recognized, and kept as raw messages otherwise.

/// Which process stream a proxy log line came from. Recorded so stderr can be
/// surfaced only when it materially helps diagnose an issue; the feed label
/// itself stays the engine name (e.g. `xray`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyStream {
    Stdout,
    Stderr,
}

/// A single parsed proxy engine log row. `time`, `level`, and `component` are
/// `None` when the source line does not match a known format; `message` then
/// holds the raw line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiProxyLogRow {
    pub engine: String,
    pub stream: ProxyStream,
    pub time: Option<String>,
    pub level: Option<String>,
    pub component: Option<String>,
    pub message: String,
}

impl TuiProxyLogRow {
    /// Parse one raw line emitted by `engine` on `stream`. Both the xray and
    /// sing-box log formats are parsed into fields; anything else is preserved as
    /// a raw message so nothing is hidden.
    pub fn parse(engine: &str, stream: ProxyStream, line: &str) -> Self {
        let mut row = Self {
            engine: engine.to_string(),
            stream,
            time: None,
            level: None,
            component: None,
            message: line.trim_end().to_string(),
        };

        if let Some(parsed) = parse_xray_line(line).or_else(|| parse_singbox_line(line)) {
            row.time = parsed.time;
            row.level = parsed.level;
            row.component = parsed.component;
            row.message = parsed.message;
        }

        row
    }

    /// Stable identity of a row used to mark a view-only clear boundary in the
    /// engine tab. Rows reload fully each tick, so the signature lets the view
    /// hide everything up to the row visible at clear time.
    pub fn signature(&self) -> String {
        let stream = match self.stream {
            ProxyStream::Stdout => "o",
            ProxyStream::Stderr => "e",
        };
        format!("{}|{stream}|{}", self.engine, self.message)
    }
}

struct ParsedLine {
    time: Option<String>,
    level: Option<String>,
    component: Option<String>,
    message: String,
}

/// Parse an xray log line of the form
/// `2026/06/06 15:10:29.760343 [Warning] core: Xray 26.3.27 started`.
/// Returns `None` when the leading `DATE TIME` timestamp is absent.
fn parse_xray_line(line: &str) -> Option<ParsedLine> {
    let trimmed = line.trim_end();
    let mut tokens = trimmed.splitn(3, ' ');
    let date = tokens.next()?;
    if !looks_like_xray_date(date) {
        return None;
    }
    let time = tokens.next()?;
    if !looks_like_xray_time(time) {
        return None;
    }
    let rest = tokens.next().unwrap_or("").trim_start();

    let timestamp = format!("{date} {time}");
    let (level, after_level) = split_level(rest);
    let (component, message) = split_component(after_level);

    Some(ParsedLine {
        time: Some(timestamp),
        level,
        component,
        message: message.to_string(),
    })
}

/// Parse a sing-box log line. With `log.timestamp` enabled (which generated
/// configs set) sing-box emits
/// `2026-06-06 15:10:29 INFO router: ...`, optionally prefixed with a timezone
/// offset token such as `+0330`. Returns `None` unless a known sing-box level
/// word is present, so xray lines and unrelated output fall through to raw.
fn parse_singbox_line(line: &str) -> Option<ParsedLine> {
    let trimmed = line.trim_end();
    let mut rest = trimmed.trim_start();

    if let Some(offset) = rest.split_whitespace().next()
        && looks_like_tz_offset(offset)
    {
        rest = rest[offset.len()..].trim_start();
    }

    let mut date = None;
    if let Some(token) = rest.split_whitespace().next()
        && looks_like_singbox_date(token)
    {
        date = Some(token.to_string());
        rest = rest[token.len()..].trim_start();
        if let Some(token) = rest.split_whitespace().next()
            && looks_like_singbox_time(token)
        {
            date = Some(format!("{} {token}", date.take().unwrap()));
            rest = rest[token.len()..].trim_start();
        }
    }

    let level_token = rest.split_whitespace().next()?;
    if !is_singbox_level(level_token) {
        return None;
    }
    rest = rest[level_token.len()..].trim_start();

    let (component, message) = split_component(rest);
    Some(ParsedLine {
        time: date,
        level: Some(level_token.to_string()),
        component,
        message: message.to_string(),
    })
}

fn looks_like_tz_offset(token: &str) -> bool {
    // +HHMM / -HHMM
    token.len() == 5
        && matches!(token.as_bytes()[0], b'+' | b'-')
        && token.as_bytes()[1..].iter().all(u8::is_ascii_digit)
}

fn looks_like_singbox_date(token: &str) -> bool {
    // YYYY-MM-DD
    token.len() == 10
        && token.as_bytes()[4] == b'-'
        && token.as_bytes()[7] == b'-'
        && token
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

fn looks_like_singbox_time(token: &str) -> bool {
    looks_like_xray_time(token)
}

fn is_singbox_level(token: &str) -> bool {
    matches!(
        token.to_ascii_lowercase().as_str(),
        "trace" | "debug" | "info" | "warn" | "warning" | "error" | "fatal" | "panic"
    )
}

/// `[Warning] rest` -> (`Some("Warning")`, `rest`). Returns the input unchanged
/// when it does not start with a `[level]` token.
fn split_level(rest: &str) -> (Option<String>, &str) {
    let rest = rest.trim_start();
    if let Some(stripped) = rest.strip_prefix('[')
        && let Some(end) = stripped.find(']')
    {
        let level = stripped[..end].trim().to_string();
        let after = stripped[end + 1..].trim_start();
        if !level.is_empty() {
            return (Some(level), after);
        }
    }
    (None, rest)
}

/// `core: message` -> (`Some("core")`, `"message"`). Only treats the prefix as a
/// component when it is a single space-free token followed by `: `.
fn split_component(body: &str) -> (Option<String>, &str) {
    if let Some(colon) = body.find(": ") {
        let candidate = &body[..colon];
        if !candidate.is_empty() && !candidate.contains(' ') {
            return (Some(candidate.to_string()), body[colon + 2..].trim_start());
        }
    }
    (None, body)
}

fn looks_like_xray_date(token: &str) -> bool {
    // YYYY/MM/DD
    token.len() == 10
        && token.as_bytes()[4] == b'/'
        && token.as_bytes()[7] == b'/'
        && token
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

fn looks_like_xray_time(token: &str) -> bool {
    // HH:MM:SS with optional .ffffff fraction
    let base = token.split('.').next().unwrap_or(token);
    base.len() == 8
        && base.as_bytes()[2] == b':'
        && base.as_bytes()[5] == b':'
        && base
            .bytes()
            .enumerate()
            .all(|(index, byte)| index == 2 || index == 5 || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_xray_warning_line_with_component() {
        let row = TuiProxyLogRow::parse(
            "xray",
            ProxyStream::Stdout,
            "2026/06/06 15:10:29.760343 [Warning] core: Xray 26.3.27 started",
        );
        assert_eq!(row.time.as_deref(), Some("2026/06/06 15:10:29.760343"));
        assert_eq!(row.level.as_deref(), Some("Warning"));
        assert_eq!(row.component.as_deref(), Some("core"));
        assert_eq!(row.message, "Xray 26.3.27 started");
    }

    #[test]
    fn parses_xray_line_without_component() {
        let row = TuiProxyLogRow::parse(
            "xray",
            ProxyStream::Stdout,
            "2026/06/06 15:10:30 [Info] connection established",
        );
        assert_eq!(row.time.as_deref(), Some("2026/06/06 15:10:30"));
        assert_eq!(row.level.as_deref(), Some("Info"));
        assert_eq!(row.component, None);
        assert_eq!(row.message, "connection established");
    }

    #[test]
    fn keeps_unparseable_line_as_raw_message() {
        let row = TuiProxyLogRow::parse("sing-box", ProxyStream::Stderr, "panic: nil map access");
        assert_eq!(row.time, None);
        assert_eq!(row.level, None);
        assert_eq!(row.component, None);
        assert_eq!(row.message, "panic: nil map access");
        assert_eq!(row.stream, ProxyStream::Stderr);
    }

    #[test]
    fn parses_singbox_info_line_with_component() {
        let row = TuiProxyLogRow::parse(
            "sing-box",
            ProxyStream::Stdout,
            "2026-06-06 15:10:29 INFO router: sniffed protocol: tls",
        );
        assert_eq!(row.time.as_deref(), Some("2026-06-06 15:10:29"));
        assert_eq!(row.level.as_deref(), Some("INFO"));
        assert_eq!(row.component.as_deref(), Some("router"));
        assert_eq!(row.message, "sniffed protocol: tls");
    }

    #[test]
    fn parses_singbox_line_with_tz_offset() {
        let row = TuiProxyLogRow::parse(
            "sing-box",
            ProxyStream::Stderr,
            "+0330 2026-06-06 15:10:29 WARN inbound/socks: listen error",
        );
        assert_eq!(row.time.as_deref(), Some("2026-06-06 15:10:29"));
        assert_eq!(row.level.as_deref(), Some("WARN"));
        assert_eq!(row.component.as_deref(), Some("inbound/socks"));
        assert_eq!(row.message, "listen error");
    }

    #[test]
    fn keeps_singbox_panic_without_level_as_raw() {
        let row = TuiProxyLogRow::parse("sing-box", ProxyStream::Stderr, "panic: nil map access");
        assert_eq!(row.time, None);
        assert_eq!(row.level, None);
        assert_eq!(row.component, None);
        assert_eq!(row.message, "panic: nil map access");
    }

    #[test]
    fn does_not_treat_multiword_prefix_as_component() {
        let row = TuiProxyLogRow::parse(
            "xray",
            ProxyStream::Stdout,
            "2026/06/06 15:10:31 [Info] rejected: bad request",
        );
        assert_eq!(row.component.as_deref(), Some("rejected"));
        assert_eq!(row.message, "bad request");
    }
}
