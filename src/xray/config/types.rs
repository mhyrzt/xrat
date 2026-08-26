use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::xray::parsing::core::{ApiObject, PolicyObject};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XrayConfig {
    pub log: LogConfig,
    pub inbounds: Vec<Inbound>,
    pub outbounds: Vec<Outbound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns: Option<XrayDnsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ApiObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<PolicyObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<RoutingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayDnsConfig {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub servers: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub hosts: BTreeMap<String, XrayDnsHostValue>,
    pub query_strategy: String,
    pub use_system_hosts: bool,
    pub disable_cache: bool,
    pub disable_fallback: bool,
    pub enable_parallel_query: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum XrayDnsHostValue {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_strategy: Option<String>,
    pub rules: Vec<RoutingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutingRule {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inbound_tag: Option<Vec<String>>,
    pub outbound_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub loglevel: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inbound {
    pub tag: String,
    pub port: u16,
    pub listen: String,
    pub protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Outbound {
    pub tag: String,
    pub protocol: String,
    pub settings: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_settings: Option<StreamSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mux: Option<Mux>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mux {
    pub enabled: bool,
    pub concurrency: i32,
    pub xudp_concurrency: i32,
    #[serde(rename = "xudpProxyUDP443")]
    pub xudp_proxy_udp443: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamSettings {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub network: String,
    #[serde(rename = "method", skip_serializing_if = "String::is_empty")]
    pub(super) method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_settings: Option<TlsSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reality_settings: Option<RealitySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_settings: Option<WsSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_settings: Option<RawSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kcp_settings: Option<KcpSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_settings: Option<GrpcSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xhttp_settings: Option<XhttpSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub httpupgrade_settings: Option<HttpUpgradeSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finalmask: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sockopt: Option<Sockopt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sockopt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dialer_proxy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TlsSettings {
    pub server_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_insecure: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<Vec<String>>,
    #[serde(rename = "echConfigList", skip_serializing_if = "Option::is_none")]
    pub ech_config_list: Option<String>,
    #[serde(
        rename = "pinnedPeerCertSha256",
        skip_serializing_if = "Option::is_none"
    )]
    pub pinned_peer_cert_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_peer_cert_by_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealitySettings {
    pub server_name: String,
    #[serde(rename = "password")]
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spider_x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mldsa65_verify: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XhttpSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(rename = "heartbeatPeriod", skip_serializing_if = "Option::is_none")]
    pub heartbeat_period: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KcpSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tti: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uplink_capacity: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downlink_capacity: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub congestion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_buffer_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_buffer_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwnd_multiplier: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_sending_window: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrpcSettings {
    pub service_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_mode: Option<bool>,
    #[serde(rename = "idle_timeout", skip_serializing_if = "Option::is_none")]
    pub idle_timeout: Option<u64>,
    #[serde(
        rename = "health_check_timeout",
        skip_serializing_if = "Option::is_none"
    )]
    pub health_check_timeout: Option<u64>,
    #[serde(
        rename = "permit_without_stream",
        skip_serializing_if = "Option::is_none"
    )]
    pub permit_without_stream: Option<bool>,
    #[serde(
        rename = "initial_windows_size",
        skip_serializing_if = "Option::is_none"
    )]
    pub initial_windows_size: Option<i64>,
    #[serde(rename = "user_agent", skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpUpgradeSettings {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(
        rename = "acceptProxyProtocol",
        skip_serializing_if = "Option::is_none"
    )]
    pub accept_proxy_protocol: Option<bool>,
}
