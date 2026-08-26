use serde::{Deserialize, Serialize};

use crate::xray::parsing::shared::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundSettingsHttp {
    pub address: Address,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundHttpServers {
    pub servers: Vec<OutboundSettingsHttp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutboundHttpConfig {
    Direct(OutboundSettingsHttp),
    Servers(OutboundHttpServers),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundSettingsShadowsocks {
    pub address: Address,
    pub port: u16,
    pub method: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uot: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uot_version: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundShadowsocksServers {
    pub servers: Vec<OutboundSettingsShadowsocks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutboundShadowsocksConfig {
    Direct(OutboundSettingsShadowsocks),
    Servers(OutboundShadowsocksServers),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundSettingsSocks {
    pub address: Address,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pass: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundSocksServers {
    pub servers: Vec<OutboundSettingsSocks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OutboundSocksConfig {
    Direct(OutboundSettingsSocks),
    Servers(OutboundSocksServers),
}
