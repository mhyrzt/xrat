use serde::Deserialize;

use super::defaults;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TestingSettings {
    pub real_delay: RealDelayTestSettings,
    pub icmp: TimeoutSettings,
    pub download: DownloadTestSettings,
    pub tcp: TimeoutSettings,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RealDelayTestSettings {
    pub url: String,
    pub timeout: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct DownloadTestSettings {
    pub url: String,
    pub timeout: u64,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct TimeoutSettings {
    pub timeout: u64,
}

impl Default for TestingSettings {
    fn default() -> Self {
        Self {
            real_delay: RealDelayTestSettings::default(),
            icmp: TimeoutSettings {
                timeout: defaults::DEFAULT_ICMP_TIMEOUT_MS,
            },
            download: DownloadTestSettings::default(),
            tcp: TimeoutSettings {
                timeout: defaults::DEFAULT_TCP_TIMEOUT_MS,
            },
        }
    }
}

impl Default for RealDelayTestSettings {
    fn default() -> Self {
        Self {
            url: defaults::DEFAULT_REAL_DELAY_TEST_URL.to_string(),
            timeout: defaults::DEFAULT_REAL_DELAY_TIMEOUT_MS,
        }
    }
}

impl Default for DownloadTestSettings {
    fn default() -> Self {
        Self {
            url: defaults::DEFAULT_DOWNLOAD_TEST_URL.to_string(),
            timeout: defaults::DEFAULT_DOWNLOAD_TIMEOUT_MS,
        }
    }
}
