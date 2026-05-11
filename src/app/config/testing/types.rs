use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TestingSettings {
    pub concurrency: i32,
    pub order: Vec<ConnectionTestStage>,
    pub failure_policy: TestFailurePolicy,
    pub real_delay: RealDelayTestSettings,
    pub icmp: IcmpTestSettings,
    pub download: DownloadTestSettings,
    pub tcp: TcpTestSettings,
    pub geoip: GeoIpTestSettings,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionTestStage {
    #[serde(alias = "ping")]
    Icmp,
    #[serde(alias = "real-delay")]
    RealDelay,
    #[serde(alias = "download-speed")]
    Download,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TestFailurePolicy {
    Continue,
    #[serde(alias = "skip-remaining")]
    SkipRemaining,
    #[serde(alias = "mark-failed")]
    MarkFailed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RealDelayTestSettings {
    pub enabled: bool,
    pub url: String,
    pub timeout: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DownloadTestSettings {
    pub enabled: bool,
    pub url: String,
    pub timeout: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct IcmpTestSettings {
    pub enabled: bool,
    pub attempts: u32,
    pub timeout: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TcpTestSettings {
    pub enabled: bool,
    pub timeout: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GeoIpTestSettings {
    pub enabled: bool,
    pub country_path: PathBuf,
    pub city_path: PathBuf,
    pub asn_path: PathBuf,
}
