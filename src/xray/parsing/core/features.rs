use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::xray::parsing::shared::DurationString;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<String>,
}

pub type StatsObject = HashMap<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeObject {
    pub tag: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortalObject {
    pub tag: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReverseObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridges: Option<Vec<BridgeObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portals: Option<Vec<PortalObject>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FakeDnsPoolObject {
    pub ip_pool: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FakeDnsObject {
    Single(FakeDnsPoolObject),
    Multiple(Vec<FakeDnsPoolObject>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricsObject {
    pub tag: String,
    pub listen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservatoryObject {
    pub subject_selector: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probe_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probe_interval: Option<DurationString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_concurrency: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingConfigObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connectivity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<DurationString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<DurationString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BurstObservatoryObject {
    pub subject_selector: Vec<String>,
    pub ping_config: PingConfigObject,
}
