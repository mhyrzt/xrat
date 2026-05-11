use std::path::PathBuf;

use serde::Deserialize;

use super::super::SecretString;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RuntimeSettings {
    pub engine: String,
    pub replace_active_session: bool,
    pub log: LogSettings,
    pub socks: SocksSettings,
    pub http: HttpSettings,
    pub shadowsocks: ShadowsocksSettings,
    pub sniffing: SniffingSettings,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct LogSettings {
    pub enabled: bool,
    pub mask: String,
    pub dir: PathBuf,
    pub dns_log: bool,
    pub level: String,
    pub keep: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SocksSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub udp: bool,
    pub auth: AuthSettings,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AuthSettings {
    pub enabled: bool,
    pub username: Option<String>,
    pub password: Option<SecretString>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct HttpSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct ShadowsocksSettings {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub method: String,
    pub password: SecretString,
    pub network: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct SniffingSettings {
    pub enabled: bool,
    pub dest_override: Vec<String>,
    pub route_only: bool,
    pub metadata_only: bool,
    pub domains_excluded: Vec<String>,
    pub ips_excluded: Vec<String>,
}
