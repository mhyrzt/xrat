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
    pub icmp_ok: Option<bool>,
    pub icmp_ms: Option<i64>,
    pub tcp_ok: Option<bool>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ok: Option<bool>,
    pub real_delay_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub connect_ms: Option<i64>,
    pub ttfb_ms: Option<i64>,
    pub http_status: Option<i64>,
    pub dial_endpoint_location: Option<String>,
    pub dial_endpoint_country: Option<String>,
    pub dial_endpoint_asn: Option<String>,
    pub dial_endpoint_geoip_source: Option<String>,
    pub dial_endpoint_fronting: Option<String>,
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
        extensions: extensions_from_raw(&config.raw_config),
        raw_config: config.raw_config.clone(),
    })
}

/// Stored config records do not persist transport extensions (`pbk`, `sid`,
/// `fp`, `flow`, `alpn`, `mode`). Recover them by re-parsing the original link
/// so runtime config generation can build REALITY/flow settings. The DB columns
/// remain authoritative for every other field.
fn extensions_from_raw(raw_config: &str) -> Option<std::collections::BTreeMap<String, String>> {
    crate::config::parse_link(raw_config)
        .ok()
        .flatten()
        .and_then(|node| node.extensions)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reality_record() -> ConfigRecord {
        ConfigRecord {
            id: 1,
            r#ref: "ref".to_string(),
            subscription_id: None,
            dedup_key: "dedup".to_string(),
            protocol: "vless".to_string(),
            address: "it.example.com".to_string(),
            port: 8080,
            username: None,
            uuid: Some("7c3b4063-d56f-4ec2-b706-4eff0828c453".to_string()),
            password: None,
            method: None,
            network: "xhttp".to_string(),
            tls: Some("reality".to_string()),
            sni: Some("www.yahoo.com".to_string()),
            host: None,
            path: Some("/".to_string()),
            name: None,
            raw_config: "vless://7c3b4063-d56f-4ec2-b706-4eff0828c453@it.example.com:8080?\
                encryption=none&security=reality&type=xhttp&path=%2F&sni=www.yahoo.com&\
                fp=chrome&pbk=test-pbk&sid=79aac70f&flow=xtls-rprx-vision#node"
                .to_string(),
            is_active: false,
            is_enabled: true,
            is_deleted: false,
            deleted_at: None,
            imported_at: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn node_from_record_recovers_reality_extensions() {
        let node = node_from_record(&reality_record()).unwrap();
        let extensions = node
            .extensions
            .expect("extensions recovered from raw_config");

        assert_eq!(extensions.get("pbk").map(String::as_str), Some("test-pbk"));
        assert_eq!(extensions.get("sid").map(String::as_str), Some("79aac70f"));
        assert_eq!(extensions.get("fp").map(String::as_str), Some("chrome"));
        assert_eq!(
            extensions.get("flow").map(String::as_str),
            Some("xtls-rprx-vision")
        );
    }

    #[test]
    fn node_from_record_tolerates_unparseable_raw_config() {
        let mut record = reality_record();
        record.raw_config = "not a valid link".to_string();
        let node = node_from_record(&record).unwrap();
        assert!(node.extensions.is_none());
    }
}
