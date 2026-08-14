use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::defaults;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default)]
pub struct DnsSettings {
    pub query_strategy: String,
    pub servers: Vec<String>,
    #[serde(skip_serializing)]
    pub hosts: BTreeMap<String, DnsHostValue>,
    pub use_system_hosts: bool,
    pub disable_cache: bool,
    pub disable_fallback: bool,
    pub enable_parallel_query: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum DnsHostValue {
    One(String),
    Many(Vec<String>),
}

impl Default for DnsSettings {
    fn default() -> Self {
        Self {
            query_strategy: defaults::DEFAULT_DNS_QUERY_STRATEGY.to_string(),
            servers: Vec::new(),
            hosts: BTreeMap::new(),
            use_system_hosts: defaults::DEFAULT_DNS_USE_SYSTEM_HOSTS,
            disable_cache: defaults::DEFAULT_DNS_DISABLE_CACHE,
            disable_fallback: defaults::DEFAULT_DNS_DISABLE_FALLBACK,
            enable_parallel_query: defaults::DEFAULT_DNS_ENABLE_PARALLEL_QUERY,
        }
    }
}
