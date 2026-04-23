use serde::{Deserialize, Serialize};

use crate::config::xray::shared::DomainStrategy;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TlsCertificateObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ocsp_stapling: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_time_loading: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_chain: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitFallbackObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_per_sec: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burst_bytes_per_sec: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HappyEyeballsObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub try_delay_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prioritize_i_pv6: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interleave: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_try: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSockoptObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SockoptObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_max_seg: Option<i32>,
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
    pub interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v6only: Option<bool>,
    #[serde(rename = "V6Only")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub v6_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_sockopt: Option<Vec<CustomSockoptObject>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TlsObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_unknown_sni: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_insecure: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cipher_suites: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificates: Option<Vec<TlsCertificateObject>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_system_root: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_session_resumption: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_peer_certificate_chain_sha256: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_key_log: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RealityObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xver: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_client_ver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_client_ver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_time_diff: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub short_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spider_x: Option<String>,
}
