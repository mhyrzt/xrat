pub(super) use super::super::*;

mod filter_and_detail;
mod list_with_tests;
mod top_and_pagination;

pub(super) fn unique_node(name: &str) -> crate::model::Node {
    use crate::model::{Node, Protocol};
    Node {
        protocol: Protocol::Vless,
        address: format!("{name}.example.com"),
        port: 443,
        username: None,
        uuid: Some(format!("uuid-{name}")),
        password: None,
        method: None,
        network: "ws".to_string(),
        tls: Some("tls".to_string()),
        sni: None,
        host: None,
        path: None,
        name: Some(name.to_string()),
        extensions: None,
        raw_config: format!(
            "vless://uuid-{name}@{name}.example.com:443?type=ws&security=tls#{name}"
        ),
    }
}

pub(super) fn node_with_protocol(protocol: &str, name: &str) -> crate::model::Node {
    use crate::model::{Node, Protocol};
    let proto = match protocol {
        "trojan" => Protocol::Trojan,
        "vless" => Protocol::Vless,
        _ => Protocol::Vless,
    };
    Node {
        protocol: proto,
        address: format!("{name}.example.com"),
        port: 443,
        username: None,
        uuid: Some(format!("uuid-{name}")),
        password: None,
        method: None,
        network: "tcp".to_string(),
        tls: Some("tls".to_string()),
        sni: None,
        host: None,
        path: None,
        name: Some(name.to_string()),
        extensions: None,
        raw_config: format!("{protocol}://uuid-{name}@{name}.example.com:443#{name}"),
    }
}
