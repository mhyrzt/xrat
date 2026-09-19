use serde_json::{Value, json};

use crate::model::Node;

use super::transport::{build_tls_and_transport, take_string};

pub(super) fn build_vmess_outbound(node: &Node) -> Result<Value, String> {
    let uuid = node
        .uuid
        .as_deref()
        .filter(|value| !value.is_empty())
        .ok_or("VMess requires UUID")?;
    uuid::Uuid::parse_str(uuid).map_err(|error| format!("VMess UUID is invalid: {error}"))?;
    let mut extensions = node.extensions.clone().unwrap_or_default();
    extensions.remove("v");
    let alter_id = take_string(&mut extensions, "aid")?
        .map(|value| {
            value
                .parse::<u16>()
                .map_err(|_| "VMess aid must be an unsigned integer".to_string())
        })
        .transpose()?;
    let security = take_string(&mut extensions, "encryption")?
        .or(take_string(&mut extensions, "scy")?)
        .or(take_string(&mut extensions, "security")?)
        .unwrap_or_else(|| "auto".to_string());
    if !matches!(
        security.as_str(),
        "auto" | "none" | "zero" | "aes-128-gcm" | "chacha20-poly1305" | "aes-128-ctr"
    ) {
        return Err(format!(
            "unsupported VMess security {security:?} for sing-box"
        ));
    }
    let packet_encoding = take_string(&mut extensions, "packet_encoding")?;
    if let Some(value) = &packet_encoding
        && !matches!(value.as_str(), "" | "packetaddr" | "xudp")
    {
        return Err(format!(
            "unsupported VMess packet_encoding {value:?} for sing-box"
        ));
    }
    let (tls, transport) = build_tls_and_transport(node, &mut extensions)?;
    if let Some(key) = extensions.keys().next() {
        return Err(format!(
            "unsupported VMess link parameter {key:?} for sing-box"
        ));
    }
    let mut outbound = json!({
        "type": "vmess", "tag": "proxy", "server": node.address,
        "server_port": node.port, "uuid": uuid, "security": security,
    });
    if let Some(alter_id) = alter_id {
        outbound["alter_id"] = json!(alter_id);
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
