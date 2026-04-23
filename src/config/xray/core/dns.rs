use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config::xray::shared::QueryStrategy;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DnsHostValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsServerObject {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_i_ps: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unexpected_i_ps: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_fallback: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_strategy: Option<QueryStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serve_stale: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serve_expired_ttl: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_query: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DnsServerEntry {
    Simple(String),
    Full(DnsServerObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<HashMap<String, DnsHostValue>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<DnsServerEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_strategy: Option<QueryStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_cache: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_fallback: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_fallback_if_match: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_parallel_query: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_system_hosts: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serve_stale: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serve_expired_ttl: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}
