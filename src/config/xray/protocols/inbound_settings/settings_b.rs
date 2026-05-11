use serde::{Deserialize, Serialize};

use super::super::clients::*;
use super::super::common::FallbackObject;
use crate::config::xray::shared::{Address, Network, StringMap};

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
