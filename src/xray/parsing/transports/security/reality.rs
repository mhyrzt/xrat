use serde::{Deserialize, Serialize};

use super::LimitFallbackObject;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealityObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xver: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_client_ver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_client_ver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_time_diff: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mldsa65_seed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_fallback_upload: Option<LimitFallbackObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_fallback_download: Option<LimitFallbackObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mldsa65_verify: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spider_x: Option<String>,
}
