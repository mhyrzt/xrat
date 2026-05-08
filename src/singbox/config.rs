use serde::{Deserialize, Serialize};
use url::Url;

use crate::model::{Node, Protocol};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxConfig {
    pub log: SingboxLogConfig,
    pub inbounds: Vec<SingboxInbound>,
    pub outbounds: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxLogConfig {
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxInbound {
    #[serde(rename = "type")]
    pub kind: String,
    pub tag: String,
    pub listen: String,
    pub listen_port: u16,
}

pub fn generate_parse_config(node: &Node, local_port: u16) -> Result<SingboxConfig, String> {
    let outbound = match node.protocol {
        Protocol::Hy2 => build_hy2_outbound(node)?,
        _ => {
            return Err(format!(
                "protocol {} is not implemented for sing-box output",
                node.protocol
            ));
        }
    };

    Ok(SingboxConfig {
        log: SingboxLogConfig {
            level: "warn".to_string(),
        },
        inbounds: vec![SingboxInbound {
            kind: "socks".to_string(),
            tag: "socks-in".to_string(),
            listen: "127.0.0.1".to_string(),
            listen_port: local_port,
        }],
        outbounds: vec![
            outbound,
            serde_json::json!({"type": "direct", "tag": "direct"}),
        ],
    })
}

fn build_hy2_outbound(node: &Node) -> Result<serde_json::Value, String> {
    let parsed = Url::parse(&node.raw_config).map_err(|error| error.to_string())?;
    let mut outbound = serde_json::json!({
        "type": "hysteria2",
        "tag": "proxy",
        "server": node.address,
        "server_port": node.port,
        "password": node.password.as_deref().unwrap_or_default(),
        "tls": {
            "enabled": true,
            "server_name": node.sni.as_deref().unwrap_or(&node.address)
        }
    });

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "insecure" => {
                if matches!(value.as_ref(), "1" | "true") {
                    outbound["tls"]["insecure"] = serde_json::json!(true);
                }
            }
            "alpn" => {
                let alpn = split_csv(value.as_ref());
                if !alpn.is_empty() {
                    outbound["tls"]["alpn"] = serde_json::json!(alpn);
                }
            }
            "obfs" => {
                if value == "salamander" {
                    outbound["obfs"] = serde_json::json!({
                        "type": "salamander",
                    });
                }
            }
            "obfs-password" => {
                if !value.is_empty() {
                    if outbound.get("obfs").is_none() {
                        outbound["obfs"] = serde_json::json!({"type": "salamander"});
                    }
                    outbound["obfs"]["password"] = serde_json::json!(value.as_ref());
                }
            }
            "upmbps" => {
                if let Ok(v) = value.parse::<u32>() {
                    outbound["up_mbps"] = serde_json::json!(v);
                }
            }
            "downmbps" => {
                if let Ok(v) = value.parse::<u32>() {
                    outbound["down_mbps"] = serde_json::json!(v);
                }
            }
            _ => {}
        }
    }

    Ok(outbound)
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::generate_parse_config;
    use crate::model::{Node, Protocol};

    #[test]
    fn generates_hy2_singbox_config_with_optional_fields() {
        let node = Node {
            protocol: Protocol::Hy2,
            address: "hy2.example.com".to_string(),
            port: 443,
            username: None,
            uuid: None,
            password: Some("secret".to_string()),
            method: None,
            network: "udp".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("edge.example.com".to_string()),
            host: None,
            path: None,
            name: Some("hy2".to_string()),
            raw_config: "hy2://secret@hy2.example.com:443?sni=edge.example.com&insecure=1&alpn=h3,h2&obfs=salamander&obfs-password=pwd&upmbps=20&downmbps=80#hy2".to_string(),
        };

        let config = generate_parse_config(&node, 1080).expect("hy2 config should generate");
        let outbound = &config.outbounds[0];
        assert_eq!(outbound["type"], "hysteria2");
        assert_eq!(outbound["tls"]["insecure"], true);
        assert_eq!(outbound["tls"]["alpn"], serde_json::json!(["h3", "h2"]));
        assert_eq!(outbound["obfs"]["type"], "salamander");
        assert_eq!(outbound["obfs"]["password"], "pwd");
        assert_eq!(outbound["up_mbps"], 20);
        assert_eq!(outbound["down_mbps"], 80);
    }
}
