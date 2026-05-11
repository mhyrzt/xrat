use serde::{Deserialize, Serialize};

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
