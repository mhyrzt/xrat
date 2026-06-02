use std::path::PathBuf;

use serde::Deserialize;

use super::defaults;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct MmdbSettings {
    pub dir: PathBuf,
    pub download_url: String,
    pub timeout_secs: u64,
    pub default_editions: Vec<String>,
    pub auto_update: bool,
    pub update_interval_hours: u64,
}

impl Default for MmdbSettings {
    fn default() -> Self {
        Self {
            dir: PathBuf::from(defaults::DEFAULT_MMDB_DIR),
            download_url: defaults::DEFAULT_MMDB_DOWNLOAD_URL.to_string(),
            timeout_secs: defaults::DEFAULT_MMDB_TIMEOUT_SECS,
            default_editions: vec!["country".to_string(), "city".to_string(), "asn".to_string()],
            auto_update: defaults::DEFAULT_MMDB_AUTO_UPDATE,
            update_interval_hours: defaults::DEFAULT_MMDB_UPDATE_INTERVAL_HOURS,
        }
    }
}
