use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::tui::theme::{ThemePalette, TuiTheme};

mod colors;
mod custom;
mod selection;

pub use colors::ThemeColor;
pub use custom::CustomTheme;
pub use selection::ThemeName;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TuiBorderStyle {
    #[default]
    Plain,
    Rounded,
}

impl TuiBorderStyle {
    pub const CANONICAL_NAMES: &'static [&'static str] = &["plain", "rounded"];

    pub fn to_border_type(self) -> ratatui::widgets::BorderType {
        match self {
            Self::Plain => ratatui::widgets::BorderType::Plain,
            Self::Rounded => ratatui::widgets::BorderType::Rounded,
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "plain" => Ok(Self::Plain),
            "rounded" => Ok(Self::Rounded),
            other => Err(format!(
                "invalid border_style \"{other}\"; choose from: plain, rounded"
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plain => "plain",
            Self::Rounded => "rounded",
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct TuiSettings {
    pub theme: ThemeName,
    #[serde(default)]
    pub border_style: TuiBorderStyle,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub themes: BTreeMap<String, CustomTheme>,
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawTuiSettings {
    theme: ThemeName,
    #[serde(default)]
    border_style: TuiBorderStyle,
    #[serde(default)]
    transparent: bool,
    themes: BTreeMap<String, CustomTheme>,
}

impl<'de> Deserialize<'de> for TuiSettings {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawTuiSettings::deserialize(deserializer)?;
        let settings = Self {
            theme: raw.theme,
            border_style: raw.border_style,
            transparent: raw.transparent,
            themes: raw.themes,
        };
        settings.validate().map_err(serde::de::Error::custom)?;
        Ok(settings)
    }
}

impl TuiSettings {
    pub fn validate(&self) -> Result<(), String> {
        for name in self.themes.keys() {
            if name.parse::<TuiTheme>().is_ok() {
                return Err(format!(
                    "tui.themes.{name}: built-in names and aliases are reserved"
                ));
            }
            if !selection::valid_custom_name(name) {
                return Err(format!(
                    "tui.themes.{name}: use a lowercase name starting with a letter, followed by letters, digits or hyphens"
                ));
            }
        }
        self.resolve(&self.theme).map(|_| ())
    }

    pub fn resolve(&self, name: &ThemeName) -> Result<ThemePalette, String> {
        let mut palette = match name {
            ThemeName::Builtin(theme) => theme.palette(),
            ThemeName::Custom(name) => self.themes.get(name)
                .map(CustomTheme::palette)
                .ok_or_else(|| format!("tui.theme: unknown theme \"{name}\"; choose a built-in or define [tui.themes.{name}]"))?,
        };
        if self.transparent {
            palette.background = ratatui::style::Color::Reset;
        }
        Ok(palette)
    }

    pub fn theme_names(&self) -> Vec<String> {
        TuiTheme::CANONICAL_NAMES
            .iter()
            .map(|name| (*name).to_string())
            .chain(self.themes.keys().cloned())
            .collect()
    }
}

#[cfg(test)]
mod tests;
