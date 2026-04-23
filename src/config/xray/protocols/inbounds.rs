use serde::{Deserialize, Serialize};

use crate::config::xray::shared::{Address, PortValue};
use crate::config::xray::transports::StreamSettingsObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen: Option<Address>,
    pub port: PortValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<StreamSettingsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sniffing: Option<SniffingObject>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
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
