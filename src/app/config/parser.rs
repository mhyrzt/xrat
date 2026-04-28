use serde::Deserialize;

use crate::config::xray::ParseMode;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ParserSettings {
    pub parse_mode: ParseMode,
}

impl Default for ParserSettings {
    fn default() -> Self {
        Self {
            parse_mode: ParseMode::Strict,
        }
    }
}
