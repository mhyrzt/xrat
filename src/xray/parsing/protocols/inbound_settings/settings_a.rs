use serde::{Deserialize, Serialize};

use super::super::clients::*;
use super::super::common::{FallbackObject, HttpAccountObject};
use crate::xray::parsing::shared::{Address, Network, deserialize_optional_string_list};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsDokodemo {
    #[serde(alias = "address", skip_serializing_if = "Option::is_none")]
    pub rewrite_address: Option<Address>,
    #[serde(alias = "port", skip_serializing_if = "Option::is_none")]
    pub rewrite_port: Option<u16>,
    #[serde(
        alias = "network",
        default,
        deserialize_with = "deserialize_optional_string_list",
        skip_serializing_if = "Option::is_none"
    )]
    pub allowed_network: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_map: Option<std::collections::HashMap<String, u16>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsHttp {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<Vec<HttpAccountObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_transparent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<u32>,
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
    pub level: Option<u32>,
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
    pub user_level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsTrojan {
    pub clients: Vec<TrojanClientObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<FallbackObject>>,
}
