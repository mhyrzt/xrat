use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::super::SecretString;
use xrat_engines::xray::XrayCompatibilityPolicy;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct RuntimeSettings {
    pub engine: String,
    pub xray_compatibility: XrayCompatibilityPolicy,
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
    pub tun: TunSettings,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TunSplitMode {
    #[default]
    All,
    Blacklist,
    Whitelist,
}

impl TunSplitMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Blacklist => "blacklist",
            Self::Whitelist => "whitelist",
        }
    }

    pub fn from_config_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "all" => Some(Self::All),
            "blacklist" => Some(Self::Blacklist),
            "whitelist" => Some(Self::Whitelist),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TunAutoSetupMode {
    #[default]
    Auto,
    Always,
    Never,
}

impl TunAutoSetupMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Always => "always",
            Self::Never => "never",
        }
    }

    pub fn from_config_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Some(Self::Auto),
            "always" => Some(Self::Always),
            "never" => Some(Self::Never),
            _ => None,
        }
    }
}

impl Serialize for TunAutoSetupMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TunAutoSetupMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TunAutoSetupModeVisitor;

        impl<'de> serde::de::Visitor<'de> for TunAutoSetupModeVisitor {
            type Value = TunAutoSetupMode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#""auto", "always", "never", or a boolean"#)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(if value {
                    TunAutoSetupMode::Always
                } else {
                    TunAutoSetupMode::Never
                })
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                TunAutoSetupMode::from_config_str(value)
                    .ok_or_else(|| E::unknown_variant(value, &["auto", "always", "never"]))
            }
        }

        deserializer.deserialize_any(TunAutoSetupModeVisitor)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TunKillSwitchMode {
    #[default]
    Off,
    Auto,
    Strict,
}

impl TunKillSwitchMode {
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Auto | Self::Strict)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Auto => "auto",
            Self::Strict => "strict",
        }
    }

    pub fn from_config_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" | "false" | "disabled" | "none" => Some(Self::Off),
            "auto" | "on" | "true" | "enabled" => Some(Self::Auto),
            "strict" | "permanent" | "always" => Some(Self::Strict),
            _ => None,
        }
    }
}

impl Serialize for TunKillSwitchMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TunKillSwitchMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct TunKillSwitchModeVisitor;

        impl<'de> serde::de::Visitor<'de> for TunKillSwitchModeVisitor {
            type Value = TunKillSwitchMode;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(r#""off", "auto", "strict", or a boolean"#)
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(if value {
                    TunKillSwitchMode::Auto
                } else {
                    TunKillSwitchMode::Off
                })
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                TunKillSwitchMode::from_config_str(value)
                    .ok_or_else(|| E::unknown_variant(value, &["off", "auto", "strict"]))
            }
        }

        deserializer.deserialize_any(TunKillSwitchModeVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct TunSettings {
    pub enabled: bool,
    pub interface_name: String,
    pub mtu: u32,
    pub stack: String,
    pub address: Vec<String>,
    pub auto_route: bool,
    pub strict_route: bool,
    pub route_exclude_address: Vec<String>,
    pub split_mode: TunSplitMode,
    pub blacklist: Vec<String>,
    pub whitelist: Vec<String>,
    pub auto_setup_on_upgrade: TunAutoSetupMode,
    pub restart_daemon_on_upgrade: bool,
    pub kill_switch: TunKillSwitchMode,
    pub allow_lan: bool,
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
