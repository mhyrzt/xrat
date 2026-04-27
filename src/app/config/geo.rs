use std::path::PathBuf;

use serde::Deserialize;

use super::r#const as defaults;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GeoSettings {
    pub auto_update: bool,
    pub update_interval_hours: u64,
    pub save_dir: Option<PathBuf>,
    pub profiles: Vec<GeoProfile>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct GeoProfile {
    pub name: String,
    pub geosite: String,
    pub geoip: String,
}

impl Default for GeoSettings {
    fn default() -> Self {
        Self {
            auto_update: defaults::DEFAULT_GEO_AUTO_UPDATE,
            update_interval_hours: defaults::DEFAULT_GEO_UPDATE_INTERVAL_HOURS,
            save_dir: None,
            profiles: Vec::new(),
        }
    }
}
