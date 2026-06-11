use ratatui::style::{Color, Style};

/// Blank columns kept between a right-aligned action group and the edge.
pub const EDGE_MARGIN: usize = 2;

/// Shared label-column width for the detail/runtime panels so their
/// `Label value` columns line up to the same offset.
pub const DETAIL_LABEL_WIDTH: usize = 13;

/// Right padding for detail/runtime panels. Gives wrapped lines slack so
/// emoji-heavy config names (whose rendered width exceeds `unicode-width`'s
/// estimate) wrap before they collide with the right border.
pub const DETAIL_RIGHT_PAD: u16 = 3;

pub fn chrome_style() -> Style {
    Style::default().fg(Color::Rgb(237, 225, 205))
}

pub fn accent_style() -> Style {
    Style::default().fg(Color::Rgb(247, 181, 99))
}

pub fn muted_style() -> Style {
    Style::default().fg(Color::Rgb(148, 139, 123))
}

pub fn success_style() -> Style {
    Style::default().fg(Color::Rgb(97, 198, 128))
}

pub fn failure_style() -> Style {
    Style::default().fg(Color::Rgb(229, 96, 84))
}

pub fn warning_style() -> Style {
    Style::default().fg(Color::Rgb(204, 170, 80))
}

/// Shared severity color for app events and proxy engine logs. Maps
/// critical/fatal/panic/error to red, warn/warning to yellow, and
/// info/debug/trace (or any other recognized level) to the accent color. An
/// empty or unknown level falls back to the muted style.
pub fn severity_style(level: &str) -> Style {
    match level.trim().to_ascii_lowercase().as_str() {
        "critical" | "fatal" | "panic" | "error" | "err" => failure_style(),
        "warn" | "warning" => warning_style(),
        "info" | "debug" | "trace" | "notice" => accent_style(),
        "" => muted_style(),
        _ => accent_style(),
    }
}
