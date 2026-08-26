use serde::{Deserialize, Serialize};

use super::outbound_settings::*;
use crate::xray::parsing::shared::{Address, DomainStrategy};
use crate::xray::parsing::transports::StreamSettingsObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MuxObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xudp_concurrency: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "xudpProxyUDP443")]
    pub xudp_proxy_udp443: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxySettingsObject {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport_layer: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseOutboundObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_through: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<StreamSettingsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_settings: Option<ProxySettingsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mux: Option<MuxObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_strategy: Option<DomainStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "protocol", rename_all = "lowercase")]
pub enum OutboundObject {
    #[serde(rename = "blackhole", alias = "block")]
    Blackhole {
        #[serde(flatten)]
        base: BaseOutboundObject,
        #[serde(skip_serializing_if = "Option::is_none")]
        settings: Option<OutboundSettingsBlackhole>,
    },
    #[serde(rename = "dns")]
    Dns {
        #[serde(flatten)]
        base: BaseOutboundObject,
        #[serde(skip_serializing_if = "Option::is_none")]
        settings: Option<OutboundSettingsDns>,
    },
    #[serde(rename = "freedom", alias = "direct")]
    Freedom {
        #[serde(flatten)]
        base: BaseOutboundObject,
        #[serde(skip_serializing_if = "Option::is_none")]
        settings: Option<OutboundSettingsFreedom>,
    },
    #[serde(rename = "http")]
    Http {
        #[serde(flatten)]
        base: BaseOutboundObject,
        settings: OutboundHttpConfig,
    },
    #[serde(rename = "hysteria")]
    Hysteria {
        #[serde(flatten)]
        base: BaseOutboundObject,
        settings: OutboundSettingsHysteria,
    },
    #[serde(rename = "loopback")]
    Loopback {
        #[serde(flatten)]
        base: BaseOutboundObject,
        settings: OutboundSettingsLoopback,
    },
    #[serde(rename = "shadowsocks")]
    Shadowsocks {
        #[serde(flatten)]
        base: BaseOutboundObject,
        settings: OutboundShadowsocksConfig,
    },
    #[serde(rename = "socks")]
    Socks {
        #[serde(flatten)]
        base: BaseOutboundObject,
        settings: OutboundSocksConfig,
    },
    #[serde(rename = "trojan")]
    Trojan {
        #[serde(flatten)]
        base: BaseOutboundObject,
        settings: OutboundTrojanConfig,
    },
    #[serde(rename = "vless")]
    Vless {
        #[serde(flatten)]
        base: BaseOutboundObject,
        settings: OutboundVlessConfig,
    },
    #[serde(rename = "vmess")]
    Vmess {
        #[serde(flatten)]
        base: BaseOutboundObject,
        settings: OutboundVmessConfig,
    },
    #[serde(rename = "wireguard")]
    Wireguard {
        #[serde(flatten)]
        base: BaseOutboundObject,
        settings: OutboundSettingsWireguard,
    },
}
