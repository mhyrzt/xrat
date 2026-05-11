use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WireguardInboundPeerObject {
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_i_ps: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboundSettingsWireguard {
    pub secret_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<i32>,
    pub peers: Vec<WireguardInboundPeerObject>,
}
