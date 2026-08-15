use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::super::SecretString;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct RuntimeSettings {
    pub engine: String,
    pub replace_active_session: bool,
    pub rotation: RotationSettings,
    pub log: LogSettings,
    pub socks: SocksSettings,
    pub http: HttpSettings,
    pub shadowsocks: ShadowsocksSettings,
    pub sniffing: SniffingSettings,
    pub stats: StatsSettings,
    pub mux: MuxSettings,
    pub fragment: FragmentSettings,
    pub network: NetworkSettings,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct MuxSettings {
    pub enabled: bool,
    pub concurrency: i32,
    pub xudp_concurrency: i32,
    pub xudp_proxy_udp443: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct FragmentSettings {
    pub enabled: bool,
    pub packets_mode: String,
    pub packets: [u32; 2],
    pub length: [u32; 2],
    pub interval: [u32; 2],
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct NetworkSettings {
    pub interface: String,
    pub bind_address: String,
    pub mark: i64,
    pub listen_interface: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct StatsSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct RotationSettings {
    pub enabled: bool,
    pub interval_secs: u64,
    pub health_trigger_enabled: bool,
    pub health_failure_threshold: u32,
    pub cooldown_secs: u64,
    pub test_concurrency: i32,
    pub test_stages: Vec<String>,
    pub refresh_subscriptions: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct LogSettings {
    pub enabled: bool,
    pub mask: String,
    pub dir: PathBuf,
    pub dns_log: bool,
    pub level: String,
    pub keep: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct SocksSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub udp: bool,
    pub auth: AuthSettings,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct AuthSettings {
    pub enabled: bool,
    pub username: Option<String>,
    pub password: Option<SecretString>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct HttpSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct ShadowsocksSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub method: String,
    pub password: SecretString,
    pub network: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct SniffingSettings {
    pub enabled: bool,
    pub dest_override: Vec<String>,
    pub route_only: bool,
    pub metadata_only: bool,
    pub domains_excluded: Vec<String>,
    pub ips_excluded: Vec<String>,
}
