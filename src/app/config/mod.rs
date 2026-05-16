use std::path::{Path, PathBuf};

use serde::Deserialize;

mod database;
pub(crate) mod defaults;
mod dns;
mod geo;
mod parser;
mod paths;
mod proxy;
mod routing;
mod secret;
mod testing;

pub use database::{
    DatabaseBackend, DatabaseSettings, PostgresDatabaseSettings, SqliteDatabaseSettings,
};
pub use dns::{DnsHostValue, DnsSettings};
pub use geo::{GeoProfile, GeoSettings};
pub use parser::ParserSettings;
pub use paths::PathSettings;
pub use proxy::{
    AuthSettings, HttpSettings, LogSettings, RotationSettings, RuntimeSettings,
    ShadowsocksSettings, SniffingSettings, SocksSettings,
};
pub use routing::{RouteList, RoutingSettings};
pub use secret::{SecretError, SecretString};
pub use testing::{
    ConnectionTestStage, DownloadTestSettings, GeoIpTestSettings, IcmpTestSettings,
    RealDelayTestSettings, TcpTestSettings, TestFailurePolicy, TestingSettings,
};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AppConfig {
    pub paths: PathSettings,
    pub database: DatabaseSettings,
    pub runtime: RuntimeSettings,
    pub routing: RoutingSettings,
    pub geo: GeoSettings,
    pub dns: DnsSettings,
    pub parser: ParserSettings,
    pub testing: TestingSettings,
}

pub fn load(config_path: &Path) -> crate::app::Result<AppConfig> {
    let contents = std::fs::read_to_string(config_path)?;
    let config = toml::from_str(&contents)?;

    Ok(config)
}

pub fn resolve_config_path(base_path: &Path, configured_path: &Path) -> PathBuf {
    if configured_path.is_absolute() {
        return configured_path.to_path_buf();
    }

    base_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join(configured_path)
}

#[cfg(test)]
mod tests;
