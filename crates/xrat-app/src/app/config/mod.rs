use std::path::{Path, PathBuf};

use serde::Deserialize;

mod database;
pub mod defaults;
mod dns;
mod dns_policy;
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
pub use dns_policy::{
    DnsAction, DnsListenerSettings, DnsNetwork, DnsOutboundRule, DnsOutboundSettings,
    DnsPolicyRule, DnsResolverPath, DnsResolverSettings, FakeIpSettings,
};
pub use geo::{GeoProfile, GeoSettings};
pub use mmdb::MmdbSettings;
pub use parser::ParserSettings;
pub use path_settings::PathSettings;
pub use proxy::{
    AuthSettings, FragmentSettings, HttpSettings, LogSettings, MuxSettings, NetworkSettings,
    RotationSettings, RuntimeSettings, ShadowsocksSettings, SniffingSettings, SocksSettings,
    TunSettings, TunSplitMode,
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

#[cfg(test)]
mod tests;

mod settings;
pub use settings::AppConfig;
pub use settings::load;
pub use settings::resolve_config_path;
