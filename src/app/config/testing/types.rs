use serde::{Deserialize, Deserializer, de};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

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
    Tcp,
}

impl ConnectionTestStage {
    /// Canonical config name (snake_case), matching serde `rename_all`.
    pub fn config_name(self) -> &'static str {
        match self {
            ConnectionTestStage::Icmp => "icmp",
            ConnectionTestStage::RealDelay => "real_delay",
            ConnectionTestStage::Download => "download",
            ConnectionTestStage::Tcp => "tcp",
        }
    }

    /// Parse a config stage string, honoring the same canonical names and
    /// aliases as deserialization. This is the single source of truth for which
    /// stage names are accepted in `[testing].order` and rotation `test_stages`.
    pub fn from_config_str(value: &str) -> Option<Self> {
        serde_json::from_value(serde_json::Value::String(value.to_string())).ok()
    }
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
    pub accepted_status_codes: Option<Vec<u16>>,
    pub accepted_status_ranges: Option<Vec<HttpStatusRange>>,
    pub follow_redirects: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HttpStatusRange {
    start: u16,
    end: u16,
}

impl HttpStatusRange {
    pub(crate) fn bounds(&self) -> (u16, u16) {
        (self.start, self.end)
    }
}

impl FromStr for HttpStatusRange {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (start, end) = value
            .split_once('-')
            .ok_or_else(|| format!("status range \"{value}\" must use START-END syntax"))?;
        if end.contains('-') {
            return Err(format!(
                "status range \"{value}\" must contain exactly one hyphen"
            ));
        }

        let start = start
            .trim()
            .parse::<u16>()
            .map_err(|_| format!("status range \"{value}\" has an invalid start"))?;
        let end = end
            .trim()
            .parse::<u16>()
            .map_err(|_| format!("status range \"{value}\" has an invalid end"))?;

        if !(100..=599).contains(&start) || !(100..=599).contains(&end) {
            return Err(format!("status range \"{value}\" must stay within 100-599"));
        }
        if start > end {
            return Err(format!("status range \"{value}\" starts after it ends"));
        }

        Ok(Self { start, end })
    }
}

impl<'de> Deserialize<'de> for HttpStatusRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl fmt::Display for HttpStatusRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}-{}", self.start, self.end)
    }
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
    pub backend: GeoIpBackend,
    pub fallback: GeoIpBackend,
    pub country_path: PathBuf,
    pub city_path: PathBuf,
    pub asn_path: PathBuf,
    pub remote: RemoteGeoIpSettings,
    pub cache: GeoIpCacheSettings,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GeoIpBackend {
    Mmdb,
    #[serde(alias = "ipwhois")]
    IpWhois,
    #[serde(alias = "ipapi")]
    IpApi,
    Chain,
    None,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RemoteGeoIpSettings {
    pub provider: GeoIpRemoteProvider,
    pub endpoint: String,
    pub timeout_ms: u64,
    pub api_key: String,
    pub rate_limit_per_minute: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GeoIpRemoteProvider {
    #[serde(alias = "ipwhois")]
    IpWhois,
    #[serde(alias = "ipapi")]
    IpApi,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct GeoIpCacheSettings {
    pub enabled: bool,
    pub ttl_secs: u64,
    pub max_entries: usize,
}
