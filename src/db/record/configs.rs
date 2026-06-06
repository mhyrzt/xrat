#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfigListFilter {
    pub only_enabled: bool,
    pub only_active: bool,
    pub only_deleted: bool,
    pub include_deleted: bool,
    pub subscription_id: Option<i64>,
    pub protocol: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConfigWithLatestTest {
    pub config: ConfigRecord,
    pub test_id: Option<i64>,
    pub tcp_ok: Option<bool>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ok: Option<bool>,
    pub real_delay_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub connect_ms: Option<i64>,
    pub ttfb_ms: Option<i64>,
    pub http_status: Option<i64>,
    pub failure_kind: Option<String>,
    pub failure_reason: Option<String>,
    pub tested_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigRecord {
    pub id: i64,
    pub r#ref: String,
    pub subscription_id: Option<i64>,
    pub dedup_key: String,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub username: Option<String>,
    pub uuid: Option<String>,
    pub password: Option<String>,
    pub method: Option<String>,
    pub network: String,
    pub tls: Option<String>,
    pub sni: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub name: Option<String>,
    pub raw_config: String,
    pub is_active: bool,
    pub is_enabled: bool,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    pub imported_at: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn node_from_record(config: &ConfigRecord) -> Result<crate::model::Node, crate::app::AppError> {
    use crate::app::AppError;
    use crate::model::{Node, Protocol};

    let protocol = match config.protocol.as_str() {
        "vless" => Protocol::Vless,
        "vmess" => Protocol::Vmess,
        "ss" => Protocol::Ss,
        "trojan" => Protocol::Trojan,
        "http" => Protocol::Http,
        "socks5" => Protocol::Socks5,
        "hy2" => Protocol::Hy2,
        other => return Err(AppError::UnsupportedProtocol(other.to_string())),
    };

    Ok(Node {
        protocol,
        address: config.address.clone(),
        port: config.port as u16,
        username: config.username.clone(),
        uuid: config.uuid.clone(),
        password: config.password.clone(),
        method: config.method.clone(),
        network: config.network.clone(),
        tls: config.tls.clone(),
        sni: config.sni.clone(),
        host: config.host.clone(),
        path: config.path.clone(),
        name: config.name.clone(),
        extensions: None,
        raw_config: config.raw_config.clone(),
    })
}
