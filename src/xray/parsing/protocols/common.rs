use serde::{Deserialize, Serialize};

use crate::xray::parsing::shared::Int32Range;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpAccountObject {
    pub user: String,
    pub pass: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseTagObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FallbackObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xver: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FragmentObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packets: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<Int32Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<Int32Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_split: Option<Int32Range>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoiseObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packet: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delay: Option<Int32Range>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireguardPeerObject {
    pub endpoint: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_shared_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<u32>,
    #[serde(rename = "allowedIPs", skip_serializing_if = "Option::is_none")]
    pub allowed_ips: Option<Vec<String>>,
}
