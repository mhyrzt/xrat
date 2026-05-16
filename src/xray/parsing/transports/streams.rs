use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::security::{RealityObject, SockoptObject, TlsObject};
use crate::xray::parsing::shared::{Security, StreamNetwork};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_early_data: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_browser_forwarding: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub early_data_header_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawObject {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KcpObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tti: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uplink_capacity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downlink_capacity: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub congestion: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_buffer_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_buffer_size: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrpcObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check_timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit_without_stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_windows_size: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpUpgradeObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HysteriaObject {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XHttpSettingsObject {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinalMaskObject {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamSettingsObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<StreamNetwork>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Security>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls_settings: Option<TlsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reality_settings: Option<RealityObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_settings: Option<RawObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xhttp_settings: Option<XHttpSettingsObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kcp_settings: Option<KcpObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grpc_settings: Option<GrpcObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_settings: Option<WebSocketObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub httpupgrade_settings: Option<HttpUpgradeObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hysteria_settings: Option<HysteriaObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sockopt: Option<SockoptObject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finalmask: Option<FinalMaskObject>,
}

pub type TransportObject = StreamSettingsObject;
