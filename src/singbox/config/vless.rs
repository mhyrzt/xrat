use serde_json::{Value, json};

use crate::model::Node;

use super::transport::{build_tls_and_transport, take_string};

pub(super) fn build_vless_outbound(node: &Node) -> Result<Value, String> {
    let uuid = node
        .uuid
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or("VLESS requires UUID")?;
    uuid::Uuid::parse_str(uuid).map_err(|error| format!("VLESS UUID is invalid: {error}"))?;
    let mut extensions = node.extensions.clone().unwrap_or_default();
    let flow = take_string(&mut extensions, "flow")?;
    let encryption = take_string(&mut extensions, "encryption")?;
    if encryption.as_deref().is_some_and(|value| value != "none") {
        return Err("VLESS encryption must be none for sing-box".to_string());
    }
    let packet_encoding = take_string(&mut extensions, "packet_encoding")?;
    if let Some(value) = &packet_encoding
        && !matches!(value.as_str(), "" | "packetaddr" | "xudp")
    {
        return Err(format!(
            "unsupported VLESS packet_encoding {value:?} for sing-box"
        ));
    }
    let (tls, transport) = build_tls_and_transport(node, &mut extensions)?;
    if let Some(key) = extensions.keys().next() {
        return Err(format!(
            "unsupported VLESS link parameter {key:?} for sing-box"
        ));
    }
    let mut outbound = json!({
        "type": "vless", "tag": "proxy", "server": node.address,
        "server_port": node.port, "uuid": uuid,
    });
    if let Some(flow) = flow.filter(|value| !value.is_empty()) {
        if flow != "xtls-rprx-vision" || tls.is_none() || transport.is_some() {
            return Err(
                "VLESS flow requires xtls-rprx-vision over direct TLS or REALITY".to_string(),
            );
        }
        outbound["flow"] = json!(flow);
    }
    if let Some(packet_encoding) = packet_encoding.filter(|value| !value.is_empty()) {
        outbound["packet_encoding"] = json!(packet_encoding);
    }
    if let Some(tls) = tls {
        outbound["tls"] = tls;
    }
    if let Some(transport) = transport {
        outbound["transport"] = transport;
    }
    Ok(outbound)
}
