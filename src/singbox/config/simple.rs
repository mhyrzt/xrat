use base64::Engine;
use serde_json::{Value, json};

use crate::model::{Node, Protocol};

pub(super) fn build_outbound(node: &Node) -> Result<Value, String> {
    if let Some(key) = node
        .extensions
        .as_ref()
        .and_then(|values| values.keys().next())
    {
        return Err(format!(
            "unsupported {} link parameter {key:?} for sing-box",
            node.protocol
        ));
    }
    if !matches!(node.network.as_str(), "tcp" | "udp" | "") {
        return Err(format!(
            "unsupported {} network {:?} for sing-box",
            node.protocol, node.network
        ));
    }
    let mut outbound = json!({
        "type": match node.protocol {
            Protocol::Ss => "shadowsocks",
            Protocol::Http => "http",
            Protocol::Socks5 => "socks",
            _ => return Err(format!("unsupported simple outbound protocol {}", node.protocol)),
        },
        "tag": "proxy", "server": node.address, "server_port": node.port,
    });
    match node.protocol {
        Protocol::Ss => {
            if node.tls.is_some()
                || node.sni.is_some()
                || node.host.is_some()
                || node.path.is_some()
            {
                return Err("Shadowsocks TLS, SNI, host, and path require a plugin unsupported by this sing-box mapping".to_string());
            }
            let method = node
                .method
                .as_deref()
                .filter(|value| !value.is_empty())
                .ok_or("Shadowsocks requires method")?;
            if !matches!(
                method,
                "2022-blake3-aes-128-gcm"
                    | "2022-blake3-aes-256-gcm"
                    | "2022-blake3-chacha20-poly1305"
                    | "none"
                    | "aes-128-gcm"
                    | "aes-192-gcm"
                    | "aes-256-gcm"
                    | "chacha20-ietf-poly1305"
                    | "xchacha20-ietf-poly1305"
                    | "aes-128-ctr"
                    | "aes-192-ctr"
                    | "aes-256-ctr"
                    | "aes-128-cfb"
                    | "aes-192-cfb"
                    | "aes-256-cfb"
                    | "rc4-md5"
                    | "chacha20-ietf"
                    | "xchacha20"
            ) {
                return Err(format!(
                    "unsupported Shadowsocks method {method:?} for sing-box 1.13"
                ));
            }
            outbound["method"] = json!(method);
            let password = node
                .password
                .as_deref()
                .filter(|value| !value.is_empty())
                .ok_or("Shadowsocks requires password")?;
            if let Some(expected_bytes) = shadowsocks_2022_key_bytes(method) {
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(password.as_bytes())
                    .map_err(|error| {
                        format!("Shadowsocks method {method:?} requires a base64 key: {error}")
                    })?;
                if decoded.len() != expected_bytes {
                    return Err(format!(
                        "Shadowsocks method {method:?} requires a {expected_bytes}-byte key; got {} bytes",
                        decoded.len()
                    ));
                }
            }
            outbound["password"] = json!(password);
        }
        Protocol::Http | Protocol::Socks5 => {
            if node.host.is_some() || node.path.is_some() {
                return Err(format!(
                    "{} host/path cannot be represented by sing-box",
                    node.protocol
                ));
            }
            if let Some(username) = &node.username {
                outbound["username"] = json!(username);
            }
            if let Some(password) = &node.password {
                if node.username.is_none() {
                    return Err(format!("{} password requires username", node.protocol));
                }
                outbound["password"] = json!(password);
            }
            match (node.protocol.clone(), node.tls.as_deref()) {
                (Protocol::Http, Some("tls")) => {
                    outbound["tls"] = json!({"enabled": true, "server_name": node.sni.as_deref().unwrap_or(&node.address)});
                }
                (Protocol::Http | Protocol::Socks5, None | Some("none") | Some("")) => {
                    if node.sni.is_some() {
                        return Err("SNI requires HTTP TLS".to_string());
                    }
                }
                _ => {
                    return Err(format!(
                        "unsupported {} TLS setting {:?}",
                        node.protocol, node.tls
                    ));
                }
            }
        }
        _ => unreachable!(),
    }
    Ok(outbound)
}

fn shadowsocks_2022_key_bytes(method: &str) -> Option<usize> {
    match method {
        "2022-blake3-aes-128-gcm" => Some(16),
        "2022-blake3-aes-256-gcm" | "2022-blake3-chacha20-poly1305" => Some(32),
        _ => None,
    }
}
