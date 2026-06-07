use serde::Deserialize;

use super::{SecretString, defaults};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ServerSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub key: Option<SecretString>,
    pub pac_enabled: bool,
    pub pac_allowed_hosts: Vec<String>,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_SERVER_ENABLED,
            host: defaults::DEFAULT_SERVER_HOST.to_string(),
            port: defaults::DEFAULT_SERVER_PORT,
            key: None,
            pac_enabled: defaults::DEFAULT_SERVER_PAC_ENABLED,
            pac_allowed_hosts: defaults::DEFAULT_SERVER_PAC_ALLOWED_HOSTS
                .iter()
                .map(|host| host.to_string())
                .collect(),
        }
    }
}
