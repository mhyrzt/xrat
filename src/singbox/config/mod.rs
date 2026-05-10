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
