use serde::Deserialize;

use super::defaults;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RoutingSettings {
    pub domain_strategy: String,
    pub direct: RouteList,
    pub block: RouteList,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct RouteList {
    pub domain: Vec<String>,
    pub ip: Vec<String>,
    pub geosite: Vec<String>,
    pub geoip: Vec<String>,
}

impl Default for RoutingSettings {
    fn default() -> Self {
        Self {
            domain_strategy: defaults::DEFAULT_ROUTING_DOMAIN_STRATEGY.to_string(),
            direct: RouteList::default(),
            block: RouteList::default(),
        }
    }
}
