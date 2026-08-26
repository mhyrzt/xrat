use serde::{Deserialize, Serialize};

use super::super::clients::*;
use super::super::common::FallbackObject;
use crate::xray::parsing::shared::deserialize_optional_string_list;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsTun {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_system_routing_table: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_outbounds_interface: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsTunnel {
    #[serde(alias = "address", skip_serializing_if = "Option::is_none")]
    pub rewrite_address: Option<String>,
    #[serde(alias = "port", skip_serializing_if = "Option::is_none")]
    pub rewrite_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_map: Option<std::collections::HashMap<String, u16>>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsVless {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<VlessClientObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clients: Option<Vec<VlessClientObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decryption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<FallbackObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testseed: Option<String>,
}

impl InboundSettingsVless {
    pub fn effective_clients(&self) -> Option<&[VlessClientObject]> {
        self.clients.as_deref().or(self.users.as_deref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsVmess {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<VmessClientObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clients: Option<Vec<VmessClientObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<VmessDefaultObject>,
}

impl InboundSettingsVmess {
    pub fn effective_clients(&self) -> Option<&[VmessClientObject]> {
        self.clients.as_deref().or(self.users.as_deref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VmessDefaultObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u32>,
}
