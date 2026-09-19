use serde_json::{Value, json};

use crate::model::Node;

use super::transport::build_tls_and_transport;

pub(super) fn build_trojan_outbound(node: &Node) -> Result<Value, String> {
    let password = node
        .password
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or("Trojan requires password")?;
    let mut extensions = node.extensions.clone().unwrap_or_default();
    let (tls, transport) = build_tls_and_transport(node, &mut extensions)?;
    if let Some(key) = extensions.keys().next() {
        return Err(format!(
            "unsupported Trojan link parameter {key:?} for sing-box"
        ));
    }
    let mut outbound = json!({
        "type": "trojan", "tag": "proxy", "server": node.address,
        "server_port": node.port, "password": password,
    });
    if let Some(tls) = tls {
        outbound["tls"] = tls;
    }
    if let Some(transport) = transport {
        outbound["transport"] = transport;
    }
    Ok(outbound)
}
