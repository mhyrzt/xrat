use std::path::PathBuf;

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct PathSettings {
    pub database: Option<PathBuf>,
    pub xray: Option<PathBuf>,
    pub v2ray: Option<PathBuf>,
}
