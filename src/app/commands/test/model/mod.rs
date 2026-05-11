use super::*;

mod row;
mod status;

pub(crate) use row::TestOutputParts;
pub(crate) use status::{TestStatus, overall_status};

pub(crate) fn node_from_record(config: &ConfigRecord) -> crate::app::Result<Node> {
    let protocol = match config.protocol.as_str() {
        "vless" => crate::model::Protocol::Vless,
        "vmess" => crate::model::Protocol::Vmess,
        "ss" => crate::model::Protocol::Ss,
        "trojan" => crate::model::Protocol::Trojan,
        "http" => crate::model::Protocol::Http,
        "socks5" => crate::model::Protocol::Socks5,
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
