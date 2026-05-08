use serde::Deserialize;
use std::path::PathBuf;

use super::defaults;

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

impl Default for TestingSettings {
    fn default() -> Self {
        Self {
            concurrency: defaults::DEFAULT_TESTING_CONCURRENCY,
            order: default_connection_test_order(),
            failure_policy: TestFailurePolicy::Continue,
            real_delay: RealDelayTestSettings::default(),
            icmp: IcmpTestSettings::default(),
            download: DownloadTestSettings::default(),
            tcp: TcpTestSettings::default(),
            geoip: GeoIpTestSettings::default(),
        }
    }
}

impl Default for RealDelayTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_TEST_STAGE_ENABLED,
            url: defaults::DEFAULT_REAL_DELAY_TEST_URL.to_string(),
            timeout: defaults::DEFAULT_REAL_DELAY_TIMEOUT_MS,
        }
    }
}

fn default_connection_test_order() -> Vec<ConnectionTestStage> {
    vec![
        ConnectionTestStage::Icmp,
        ConnectionTestStage::RealDelay,
        ConnectionTestStage::Download,
    ]
}

impl Default for DownloadTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_DOWNLOAD_TEST_ENABLED,
            url: defaults::DEFAULT_DOWNLOAD_TEST_URL.to_string(),
            timeout: defaults::DEFAULT_DOWNLOAD_TIMEOUT_MS,
        }
    }
}

impl Default for IcmpTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_TEST_STAGE_ENABLED,
            attempts: defaults::DEFAULT_ICMP_ATTEMPTS,
            timeout: defaults::DEFAULT_ICMP_TIMEOUT_MS,
        }
    }
}

impl Default for TcpTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_TEST_STAGE_ENABLED,
            timeout: defaults::DEFAULT_TCP_TIMEOUT_MS,
        }
    }
}

impl Default for GeoIpTestSettings {
    fn default() -> Self {
        Self {
            enabled: defaults::DEFAULT_TEST_GEOIP_ENABLED,
            country_path: PathBuf::from(defaults::DEFAULT_TEST_GEOIP_COUNTRY_PATH),
            city_path: PathBuf::from(defaults::DEFAULT_TEST_GEOIP_CITY_PATH),
            asn_path: PathBuf::from(defaults::DEFAULT_TEST_GEOIP_ASN_PATH),
        }
    }
}
