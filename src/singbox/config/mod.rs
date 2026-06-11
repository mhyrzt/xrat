use serde::{Deserialize, Serialize};

use crate::model::{Node, Protocol};

mod hy2;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxConfig {
    pub log: SingboxLogConfig,
    pub inbounds: Vec<SingboxInbound>,
    pub outbounds: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<SingboxExperimental>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxLogConfig {
    pub level: String,
    /// sing-box omits timestamps by default. Enable them in generated configs so
    /// the TUI engine tab can parse a real time column for sing-box log lines.
    pub timestamp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxInbound {
    #[serde(rename = "type")]
    pub kind: String,
    pub tag: String,
    pub listen: String,
    pub listen_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<SingboxInboundUser>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxInboundUser {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxExperimental {
    pub clash_api: SingboxClashApi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingboxClashApi {
    pub external_controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

pub fn generate_singbox_probe_config(
    node: &Node,
    local_port: u16,
) -> Result<SingboxConfig, String> {
    let outbound = match node.protocol {
        Protocol::Hy2 => hy2::build_hy2_outbound(node)?,
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
            timestamp: true,
        },
        inbounds: vec![SingboxInbound {
            kind: "socks".to_string(),
            tag: "socks-in".to_string(),
            listen: "127.0.0.1".to_string(),
            listen_port: local_port,
            network: None,
            method: None,
            password: None,
            users: None,
        }],
        outbounds: vec![
            outbound,
            serde_json::json!({"type": "direct", "tag": "direct"}),
        ],
        experimental: None,
    })
}

pub fn generate_singbox_runtime_config(
    node: &Node,
    inbounds: Vec<SingboxInbound>,
    clash_api: Option<SingboxClashApi>,
) -> Result<SingboxConfig, String> {
    let outbound = match node.protocol {
        Protocol::Hy2 => hy2::build_hy2_outbound(node)?,
        _ => {
            return Err(format!(
                "protocol {} is not implemented for sing-box runtime output",
                node.protocol
            ));
        }
    };

    Ok(SingboxConfig {
        log: SingboxLogConfig {
            level: "warn".to_string(),
            timestamp: true,
        },
        inbounds,
        outbounds: vec![
            outbound,
            serde_json::json!({"type": "direct", "tag": "direct"}),
        ],
        experimental: clash_api.map(|clash_api| SingboxExperimental { clash_api }),
    })
}
