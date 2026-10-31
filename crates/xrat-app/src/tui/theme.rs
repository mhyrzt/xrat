use std::str::FromStr;
use std::sync::{LazyLock, RwLock};

use ratatui::style::{Color, Style};
use ratatui::widgets::BorderType;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Blank columns kept between a right-aligned action group and the edge.
pub const EDGE_MARGIN: usize = 2;

/// Shared label-column width for the detail/runtime panels so their
/// `Label value` columns line up to the same offset.
pub const DETAIL_LABEL_WIDTH: usize = 13;

/// Right padding for detail/runtime panels. Gives wrapped lines slack so
/// emoji-heavy config names (whose rendered width exceeds `unicode-width`'s
/// estimate) wrap before they collide with the right border.
pub const DETAIL_RIGHT_PAD: u16 = 3;

static ACTIVE_PALETTE: LazyLock<RwLock<ThemePalette>> =
    LazyLock::new(|| RwLock::new(TuiTheme::Default.palette()));

static ACTIVE_BORDER_TYPE: LazyLock<RwLock<BorderType>> =
    LazyLock::new(|| RwLock::new(BorderType::Plain));

#[cfg(test)]
pub static TEST_THEME_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Built-in color themes supported by the TUI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum TuiTheme {
    #[default]
    Default = 0,
    Ansi = 1,
    Catppuccin = 2,
    Dracula = 3,
    Gruvbox = 4,
    Nord = 5,
    TokyoNight = 6,
    Light = 7,
}

impl TuiTheme {
    pub const CANONICAL_NAMES: &'static [&'static str] = &[
        "default",
        "ansi",
        "catppuccin",
        "dracula",
        "gruvbox",
        "nord",
        "tokyo-night",
        "light",
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Ansi => "ansi",
            Self::Catppuccin => "catppuccin",
            Self::Dracula => "dracula",
            Self::Gruvbox => "gruvbox",
            Self::Nord => "nord",
            Self::TokyoNight => "tokyo-night",
            Self::Light => "light",
        }
    }

    pub fn palette(self) -> ThemePalette {
        match self {
            Self::Default => ThemePalette {
                selection_foreground: Color::Rgb(237, 225, 205),
                background: Color::Rgb(28, 25, 22),
                foreground: Color::Rgb(237, 225, 205),
                accent: Color::Rgb(247, 181, 99),
                secondary: Color::Rgb(130, 195, 225),
                muted: Color::Rgb(160, 152, 137),
                success: Color::Rgb(97, 198, 128),
                failure: Color::Rgb(233, 120, 109),
                warning: Color::Rgb(204, 170, 80),
                download: Color::Rgb(120, 180, 235),
                upload: Color::Rgb(210, 130, 220),
                selection_background: Color::Rgb(58, 48, 38),
            },
            Self::Ansi => ThemePalette {
                selection_foreground: Color::White,
                background: Color::Black,
                foreground: Color::Reset,
                accent: Color::Cyan,
                secondary: Color::Blue,
                muted: Color::DarkGray,
                success: Color::Green,
                failure: Color::Red,
                warning: Color::Yellow,
                download: Color::Blue,
                upload: Color::Magenta,
                selection_background: Color::Indexed(8),
            },
            Self::Catppuccin => ThemePalette {
                selection_foreground: Color::Rgb(205, 214, 244),
                background: Color::Rgb(30, 30, 46),
                foreground: Color::Rgb(205, 214, 244),
                accent: Color::Rgb(143, 184, 250),
                secondary: Color::Rgb(203, 167, 247),
                muted: Color::Rgb(177, 182, 200),
                success: Color::Rgb(166, 227, 161),
                failure: Color::Rgb(245, 157, 181),
                warning: Color::Rgb(249, 226, 175),
                download: Color::Rgb(116, 199, 236),
                upload: Color::Rgb(245, 194, 231),
                selection_background: Color::Rgb(69, 71, 90),
            },
            Self::Dracula => ThemePalette {
                selection_foreground: Color::Rgb(248, 248, 242),
                background: Color::Rgb(40, 42, 54),
                foreground: Color::Rgb(248, 248, 242),
                accent: Color::Rgb(201, 167, 250),
                secondary: Color::Rgb(139, 233, 253),
                muted: Color::Rgb(172, 182, 210),
                success: Color::Rgb(80, 250, 123),
                failure: Color::Rgb(255, 154, 154),
                warning: Color::Rgb(241, 250, 140),
                download: Color::Rgb(139, 233, 253),
                upload: Color::Rgb(255, 147, 209),
                selection_background: Color::Rgb(68, 71, 90),
            },
            Self::Gruvbox => ThemePalette {
                selection_foreground: Color::Rgb(235, 219, 178),
                background: Color::Rgb(40, 40, 40),
                foreground: Color::Rgb(235, 219, 178),
                accent: Color::Rgb(254, 128, 25),
                secondary: Color::Rgb(142, 192, 124),
                muted: Color::Rgb(171, 160, 148),
                success: Color::Rgb(184, 187, 38),
                failure: Color::Rgb(252, 123, 108),
                warning: Color::Rgb(250, 189, 47),
                download: Color::Rgb(136, 169, 156),
                upload: Color::Rgb(214, 141, 161),
                selection_background: Color::Rgb(60, 56, 54),
            },
            Self::Nord => ThemePalette {
                selection_foreground: Color::Rgb(236, 239, 244),
                background: Color::Rgb(46, 52, 64),
                foreground: Color::Rgb(236, 239, 244),
                accent: Color::Rgb(136, 192, 208),
                secondary: Color::Rgb(150, 177, 203),
                muted: Color::Rgb(166, 174, 188),
                success: Color::Rgb(163, 190, 140),
                failure: Color::Rgb(216, 159, 164),
                warning: Color::Rgb(235, 203, 139),
                download: Color::Rgb(143, 188, 187),
                upload: Color::Rgb(195, 165, 189),
                selection_background: Color::Rgb(59, 66, 82),
            },
            Self::TokyoNight => ThemePalette {
                selection_foreground: Color::Rgb(192, 202, 245),
                background: Color::Rgb(26, 27, 38),
                foreground: Color::Rgb(192, 202, 245),
                accent: Color::Rgb(122, 162, 247),
                secondary: Color::Rgb(187, 154, 247),
                muted: Color::Rgb(154, 165, 206),
                success: Color::Rgb(158, 206, 106),
                failure: Color::Rgb(247, 118, 142),
                warning: Color::Rgb(224, 175, 104),
                download: Color::Rgb(125, 207, 255),
                upload: Color::Rgb(187, 154, 247),
                selection_background: Color::Rgb(41, 46, 66),
            },
            Self::Light => ThemePalette {
                selection_foreground: Color::Rgb(76, 79, 105),
                background: Color::Rgb(239, 241, 245),
                foreground: Color::Rgb(76, 79, 105),
                accent: Color::Rgb(27, 90, 217),
                secondary: Color::Rgb(127, 53, 224),
                muted: Color::Rgb(96, 98, 118),
                success: Color::Rgb(45, 113, 30),
                failure: Color::Rgb(198, 14, 54),
                warning: Color::Rgb(138, 88, 18),
                download: Color::Rgb(3, 106, 147),
                upload: Color::Rgb(127, 53, 224),
                selection_background: Color::Rgb(220, 224, 232),
            },
        }
    }
}

impl FromStr for TuiTheme {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "default" | "amber" | "warm" => Ok(Self::Default),
            "ansi" | "terminal" => Ok(Self::Ansi),
            "catppuccin" | "mocha" | "catppuccin-mocha" => Ok(Self::Catppuccin),
            "dracula" => Ok(Self::Dracula),
            "gruvbox" | "gruvbox-dark" => Ok(Self::Gruvbox),
            "nord" => Ok(Self::Nord),
            "tokyo-night" | "tokyonight" => Ok(Self::TokyoNight),
            "light" | "latte" | "solarized-light" => Ok(Self::Light),
            other => Err(format!(
                "unknown theme \"{other}\"; accepted values: {}",
                Self::CANONICAL_NAMES.join(", ")
            )),
        }
    }
}

impl Serialize for TuiTheme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TuiTheme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        Self::from_str(&text).map_err(serde::de::Error::custom)
    }
}

/// Color set assigned to TUI UI components.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePalette {
    pub background: Color,
    pub selection_foreground: Color,
    pub foreground: Color,
    pub accent: Color,
    pub secondary: Color,
    pub muted: Color,
    pub success: Color,
    pub failure: Color,
    pub warning: Color,
    pub download: Color,
    pub upload: Color,
    pub selection_background: Color,
}

pub fn set_palette(palette: ThemePalette) {
    *ACTIVE_PALETTE.write().expect("theme palette lock poisoned") = palette;
}

#[cfg(test)]
pub fn set_active(theme: TuiTheme) {
    set_palette(theme.palette());
}

pub fn palette() -> ThemePalette {
    *ACTIVE_PALETTE.read().expect("theme palette lock poisoned")
}

pub fn surface_style() -> Style {
    let palette = palette();
    Style::default()
        .fg(palette.foreground)
        .bg(palette.background)
}

pub fn clear(frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
    frame.render_widget(ratatui::widgets::Clear, area);
    frame.render_widget(
        ratatui::widgets::Block::default().style(surface_style()),
        area,
    );
}

pub fn chrome_style() -> Style {
    Style::default().fg(palette().foreground)
}

pub fn accent_style() -> Style {
    Style::default().fg(palette().accent)
}

pub fn secondary_style() -> Style {
    Style::default().fg(palette().secondary)
}

pub fn muted_style() -> Style {
    Style::default().fg(palette().muted)
}

pub fn success_style() -> Style {
    Style::default().fg(palette().success)
}

pub fn failure_style() -> Style {
    Style::default().fg(palette().failure)
}

pub fn warning_style() -> Style {
    Style::default().fg(palette().warning)
}

pub fn selection_style() -> Style {
    Style::default()
        .fg(palette().selection_foreground)
        .bg(palette().selection_background)
}

/// Border style for cards and modals. Focused panels get the primary accent,
/// while unfocused panels stay subtle and muted so content remains readable.
pub fn border_style(focused: bool) -> Style {
    if focused {
        accent_style()
    } else {
        muted_style()
    }
}

/// Preferred border type for dashboard cards and modal windows.
/// Configurable: Plain (classic square corners) or Rounded (smooth corners).
pub fn border_type() -> BorderType {
    *ACTIVE_BORDER_TYPE.read().unwrap_or_else(|e| e.into_inner())
}

/// Set active border type for cards and modals across the application.
pub fn set_border_type(border_type: BorderType) {
    if let Ok(mut lock) = ACTIVE_BORDER_TYPE.write() {
        *lock = border_type;
    }
}

/// Style for latency thresholds: green (<150ms), yellow (150..400ms), red (>400ms).
pub fn latency_style(ms: u64) -> Style {
    if ms < 150 {
        success_style()
    } else if ms < 400 {
        warning_style()
    } else {
        failure_style()
    }
}

/// Parse string like "12 ms" or "350ms" and color by latency threshold.
pub fn latency_text_style(text: &str) -> Style {
    let digits: String = text.chars().take_while(|c| c.is_ascii_digit()).collect();
    if let Ok(ms) = digits.parse::<u64>() {
        latency_style(ms)
    } else {
        muted_style()
    }
}

pub fn available_style(available: bool) -> Style {
    if available {
        success_style()
    } else {
        muted_style()
    }
}

pub fn optional_latency_style(ms: Option<i64>) -> Style {
    ms.and_then(|ms| u64::try_from(ms).ok())
        .map(latency_style)
        .unwrap_or_else(muted_style)
}

/// Dynamic style for latency and throughput metrics in graphs and tables.
pub fn series_style(label: &str) -> Style {
    let p = palette();
    let color = match label {
        "icmp" => p.success,
        "tcp" => p.warning,
        "real-delay" => p.accent,
        "download" => p.download,
        "upload" => p.upload,
        _ => p.foreground,
    };
    Style::default().fg(color)
}

/// Shared severity color for app events and proxy engine logs. Maps
/// critical/fatal/panic/error to red, warn/warning to yellow, and
/// info/debug/trace (or any other recognized level) to secondary color. An
/// empty or unknown level falls back to the muted style.
pub fn severity_style(level: &str) -> Style {
    match level.trim().to_ascii_lowercase().as_str() {
        "critical" | "fatal" | "panic" | "error" | "err" => failure_style(),
        "warn" | "warning" => warning_style(),
        "info" | "debug" | "trace" | "notice" => secondary_style(),
        "" => muted_style(),
        _ => secondary_style(),
    }
}

#[cfg(test)]
#[path = "theme_tests.rs"]
mod tests;
