use serde::Deserialize;

use super::{SecretString, defaults};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub key: Option<SecretString>,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_SERVER_ENABLED,
            host: defaults::DEFAULT_SERVER_HOST.to_string(),
            port: defaults::DEFAULT_SERVER_PORT,
            key: None,
        }
    }
}
