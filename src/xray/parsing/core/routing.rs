use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::xray::parsing::shared::{DurationString, PortValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingWebhookObject {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deduplication: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<PortValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_port: Option<PortValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_port: Option<PortValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ip: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_ip: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_tag: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vless_route: Option<PortValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbound_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balancer_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<RoutingWebhookObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingCostObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regexp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#match: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingStrategySettingsObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_rtt: Option<DurationString>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baselines: Option<Vec<DurationString>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub costs: Option<Vec<RoutingCostObject>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingStrategyObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<RoutingStrategySettingsObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BalancerObject {
    pub tag: String,
    pub selector: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<RoutingStrategyObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<RuleObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balancers: Option<Vec<BalancerObject>>,
}
