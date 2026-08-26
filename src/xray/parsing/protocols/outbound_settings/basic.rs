use serde::{Deserialize, Serialize};

use super::super::common::{FragmentObject, NoiseObject};
use crate::xray::parsing::shared::{
    Address, DomainStrategy, Int32Range, Network, PortValue, deserialize_optional_string_list,
};

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
    pub rewrite_network: Option<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewrite_address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rewrite_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<DnsOutboundRule>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_ip_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_types: Option<PortValue>,
}

impl OutboundSettingsDns {
    pub fn effective_network(&self) -> Option<&Network> {
        self.network.as_ref().or(self.rewrite_network.as_ref())
    }

    pub fn effective_address(&self) -> Option<&Address> {
        self.address.as_ref().or(self.rewrite_address.as_ref())
    }

    pub fn effective_port(&self) -> Option<u16> {
        self.port.or(self.rewrite_port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsOutboundRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(rename = "qType", skip_serializing_if = "Option::is_none")]
    pub q_type: Option<PortValue>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_list",
        skip_serializing_if = "Option::is_none"
    )]
    pub domain: Option<Vec<String>>,
    #[serde(rename = "rCode", skip_serializing_if = "Option::is_none")]
    pub r_code: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutboundSettingsFreedom {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_strategy: Option<DomainStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_strategy: Option<DomainStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fragment: Option<FragmentObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise: Option<NoiseObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noises: Option<Vec<NoiseObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_protocol: Option<u32>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_list",
        skip_serializing_if = "Option::is_none"
    )]
    pub ips_blocked: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_rules: Option<Vec<FreedomFinalRule>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FreedomFinalRule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_list",
        skip_serializing_if = "Option::is_none"
    )]
    pub network: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<PortValue>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_list",
        skip_serializing_if = "Option::is_none"
    )]
    pub ip: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_delay: Option<Int32Range>,
}
