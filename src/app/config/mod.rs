use std::path::{Path, PathBuf};

use serde::Deserialize;

mod database;
pub(crate) mod defaults;
mod dns;
mod editor;
mod geo;
mod mmdb;
mod parser;
mod path_settings;
mod proxy;
mod routing;
mod secret;
mod server;
mod subscriptions;
mod testing;

pub(crate) use editor::{
    ConfigEditSession, EditableSetting, SettingEffect, SettingKind, SettingValue,
    update_runtime_binary_path,
};

pub use database::{
    DatabaseBackend, DatabaseSettings, PostgresDatabaseSettings, SqliteDatabaseSettings,
};
pub use dns::{DnsHostValue, DnsSettings};
pub use geo::{GeoProfile, GeoSettings};
pub use mmdb::MmdbSettings;
pub use parser::ParserSettings;
pub use path_settings::PathSettings;
pub use proxy::{
    AuthSettings, FragmentSettings, HttpSettings, LogSettings, MuxSettings, NetworkSettings,
    RotationSettings, RuntimeSettings, ShadowsocksSettings, SniffingSettings, SocksSettings,
};
pub use routing::{RouteList, RoutingSettings};
pub use secret::{SecretError, SecretString};
pub use server::ServerSettings;
pub use subscriptions::SubscriptionSettings;
pub use testing::{
    ConnectionTestStage, DownloadTestSettings, GeoIpBackend, GeoIpCacheSettings,
    GeoIpRemoteProvider, GeoIpTestSettings, HttpStatusRange, IcmpTestSettings,
    RealDelayTestSettings, RemoteGeoIpSettings, TcpTestSettings, TestFailurePolicy,
    TestingSettings,
};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AppConfig {
    pub paths: PathSettings,
    pub database: DatabaseSettings,
    pub runtime: RuntimeSettings,
    pub subscriptions: SubscriptionSettings,
    pub routing: RoutingSettings,
    pub geo: GeoSettings,
    pub mmdb: MmdbSettings,
    pub dns: DnsSettings,
    pub parser: ParserSettings,
    pub testing: TestingSettings,
    pub server: ServerSettings,
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
