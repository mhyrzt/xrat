use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::inbound_settings::*;
use crate::xray::parsing::shared::{Address, PortValue};
use crate::xray::parsing::transports::StreamSettingsObject;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<PortValue>,
    pub protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<InboundSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<StreamSettingsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sniffing: Option<SniffingObject>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum InboundSettings {
    Dokodemo(InboundSettingsDokodemo),
    Http(InboundSettingsHttp),
    Hysteria(InboundSettingsHysteria),
    Shadowsocks(InboundSettingsShadowsocks),
    Socks(InboundSettingsSocks),
    Trojan(InboundSettingsTrojan),
    Tun(InboundSettingsTun),
    Tunnel(InboundSettingsTunnel),
    Vless(InboundSettingsVless),
    Vmess(InboundSettingsVmess),
    Wireguard(InboundSettingsWireguard),
    Unknown(Value),
}

impl InboundObject {
    pub(crate) fn has_unknown_protocol(&self) -> bool {
        !matches!(
            self.protocol.as_str(),
            "dokodemo-door"
                | "http"
                | "hysteria"
                | "shadowsocks"
                | "socks"
                | "mixed"
                | "trojan"
                | "tun"
                | "tunnel"
                | "vless"
                | "vmess"
                | "wireguard"
        )
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawInboundObject {
    listen: Option<Address>,
    port: Option<PortValue>,
    protocol: String,
    settings: Option<Value>,
    stream_settings: Option<StreamSettingsObject>,
    tag: Option<String>,
    sniffing: Option<SniffingObject>,
}

impl<'de> Deserialize<'de> for InboundObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawInboundObject::deserialize(deserializer)?;
        let settings = raw
            .settings
            .map(|value| {
                let parsed = match raw.protocol.as_str() {
                    "dokodemo-door" => serde_json::from_value(value).map(InboundSettings::Dokodemo),
                    "http" => serde_json::from_value(value).map(InboundSettings::Http),
                    "hysteria" => serde_json::from_value(value).map(InboundSettings::Hysteria),
                    "shadowsocks" => {
                        serde_json::from_value(value).map(InboundSettings::Shadowsocks)
                    }
                    "socks" | "mixed" => serde_json::from_value(value).map(InboundSettings::Socks),
                    "trojan" => serde_json::from_value(value).map(InboundSettings::Trojan),
                    "tun" => serde_json::from_value(value).map(InboundSettings::Tun),
                    "tunnel" => serde_json::from_value(value).map(InboundSettings::Tunnel),
                    "vless" => serde_json::from_value(value).map(InboundSettings::Vless),
                    "vmess" => serde_json::from_value(value).map(InboundSettings::Vmess),
                    "wireguard" => serde_json::from_value(value).map(InboundSettings::Wireguard),
                    _ => Ok(InboundSettings::Unknown(value)),
                };
                parsed.map_err(serde::de::Error::custom)
            })
            .transpose()?;

        Ok(Self {
            listen: raw.listen,
            port: raw.port,
            protocol: raw.protocol,
            settings,
            stream_settings: raw.stream_settings,
            tag: raw.tag,
            sniffing: raw.sniffing,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SniffingObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_override: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains_excluded: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ips_excluded: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_only: Option<bool>,
}
