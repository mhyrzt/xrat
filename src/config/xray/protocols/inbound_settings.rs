use serde::{Deserialize, Serialize};

use super::clients::*;
use super::common::{FallbackObject, HttpAccountObject};
use crate::config::xray::shared::{Address, Network, StringMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsDokodemo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsHttp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<HttpAccountObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_transparent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsHysteria {
    pub version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clients: Option<Vec<HysteriaClientObject>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsShadowsocks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clients: Option<Vec<ShadowsocksClientObject>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsSocks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<HttpAccountObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsTrojan {
    pub clients: Vec<TrojanClientObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<FallbackObject>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsTun {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "MTU")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu_upper: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<i32>,
    #[serde(rename = "UserLevel")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level_upper: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsTunnel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_map: Option<StringMap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsVless {
    pub clients: Vec<VlessClientObject>,
    pub decryption: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<FallbackObject>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsVmess {
    pub clients: Vec<VmessClientObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<VmessDefaultObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmessDefaultObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireguardInboundPeerObject {
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_i_ps: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsWireguard {
    pub secret_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<i32>,
    pub peers: Vec<WireguardInboundPeerObject>,
}
