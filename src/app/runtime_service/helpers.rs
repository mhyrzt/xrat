use super::*;

pub(super) fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

pub(super) fn node_from_record(config: &ConfigRecord) -> crate::app::Result<Node> {
    let protocol = match config.protocol.as_str() {
        "vless" => Protocol::Vless,
        "vmess" => Protocol::Vmess,
        "ss" => Protocol::Ss,
        "trojan" => Protocol::Trojan,
        "http" => Protocol::Http,
        "socks5" => Protocol::Socks5,
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

pub(super) fn connect_host_for_bind_host(host: &str) -> String {
    match host {
        "0.0.0.0" => "127.0.0.1".to_string(),
        "::" => "::1".to_string(),
        _ => host.to_string(),
    }
}
