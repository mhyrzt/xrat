mod default_values;
mod types;

pub use types::{
    ConnectionTestStage, DownloadTestSettings, GeoIpBackend, GeoIpCacheSettings,
    GeoIpRemoteProvider, GeoIpTestSettings, IcmpTestSettings, RealDelayTestSettings,
    RemoteGeoIpSettings, TcpTestSettings, TestFailurePolicy, TestingSettings,
};
