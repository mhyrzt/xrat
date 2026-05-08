use serde_json::json;

use super::stream::build_stream_settings;
use super::types::Outbound;
use crate::model::{Node, Protocol};

pub(super) fn node_to_outbound(node: &Node, tag: &str) -> Result<Outbound, String> {
    let protocol = node.protocol.as_str().to_string();
    let settings = build_outbound_settings(node)?;
    let stream_settings = build_stream_settings(node)?;

    Ok(Outbound {
        tag: tag.to_string(),
        protocol,
        settings,
        stream_settings,
    })
}

fn build_outbound_settings(node: &Node) -> Result<serde_json::Value, String> {
    match node.protocol {
        Protocol::Vless => {
            let uuid = node.uuid.as_ref().ok_or("vless requires uuid")?;
            Ok(json!({
                "vnext": [{
                    "address": node.address,
                    "port": node.port,
                    "users": [{
                        "id": uuid,
                        "encryption": "none"
                    }]
                }]
            }))
        }
        Protocol::Vmess => {
            let uuid = node.uuid.as_ref().ok_or("vmess requires uuid")?;
            Ok(json!({
                "vnext": [{
                    "address": node.address,
                    "port": node.port,
                    "users": [{
                        "id": uuid,
                        "alterId": 0,
                        "security": "auto"
                    }]
                }]
            }))
        }
        Protocol::Trojan => {
            let password = node.password.as_ref().ok_or("trojan requires password")?;
            Ok(json!({
                "servers": [{
                    "address": node.address,
                    "port": node.port,
                    "password": password
                }]
            }))
        }
        Protocol::Ss => {
            let password = node
                .password
                .as_ref()
                .ok_or("shadowsocks requires password")?;
            let method = node.method.as_ref().ok_or("shadowsocks requires method")?;
            Ok(json!({
                "servers": [{
                    "address": node.address,
                    "port": node.port,
                    "method": method,
                    "password": password
                }]
            }))
        }
        Protocol::Socks5 => {
            let mut server = json!({
                "address": node.address,
                "port": node.port
            });
            if let Some(username) = &node.username {
                if let Some(password) = &node.password {
                    server["users"] = json!([{
                        "user": username,
                        "pass": password
                    }]);
                }
            }
            Ok(json!({
                "servers": [server]
            }))
        }
        Protocol::Http => {
            let mut server = json!({
                "address": node.address,
                "port": node.port
            });
            if let Some(username) = &node.username {
                if let Some(password) = &node.password {
                    server["users"] = json!([{
                        "user": username,
                        "pass": password
                    }]);
                }
            }
            Ok(json!({
                "servers": [server]
            }))
        }
        Protocol::Hy2 => Err("hysteria2/hy2 is not supported by xray config generator".to_string()),
    }
}
