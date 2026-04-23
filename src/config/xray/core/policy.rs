use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelPolicyObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handshake: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conn_idle: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uplink_only: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downlink_only: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_user_uplink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_user_downlink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_user_online: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemPolicyObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_inbound_uplink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_inbound_downlink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_outbound_uplink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats_outbound_downlink: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub levels: Option<HashMap<String, LevelPolicyObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPolicyObject>,
}
