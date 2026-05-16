use serde::{Deserialize, Serialize};

use super::super::common::{FragmentObject, NoiseObject};
use crate::xray::parsing::shared::{Address, DomainStrategy, Network};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundSettingsBlackhole {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<BlackholeResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlackholeResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundSettingsDns {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_ip_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_types: Option<Vec<i32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundSettingsFreedom {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_strategy: Option<DomainStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment: Option<FragmentObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noises: Option<Vec<NoiseObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_protocol: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ips_blocked: Option<Vec<String>>,
}
