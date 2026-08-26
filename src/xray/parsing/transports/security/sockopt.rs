use serde::{Deserialize, Serialize};

use crate::xray::parsing::shared::DomainStrategy;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HappyEyeballsObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub try_delay_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "prioritizeIPv6")]
    pub prioritize_ipv6: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interleave: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_try: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSockoptObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SockoptObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_max_seg: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_window_clamp: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_fast_open: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tproxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_strategy: Option<DomainStrategy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub happy_eyeballs: Option<HappyEyeballsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dialer_proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_proxy_protocol: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_keep_alive_interval: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_keep_alive_idle: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_user_timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_congestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcpcongestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub penetrate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "tcpMptcp")]
    pub tcp_mptcp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_port_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "trustedXForwardedFor")]
    pub trusted_x_forwarded_for: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v6only: Option<bool>,
    #[serde(rename = "V6Only")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v6_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_sockopt: Option<Vec<CustomSockoptObject>>,
}
