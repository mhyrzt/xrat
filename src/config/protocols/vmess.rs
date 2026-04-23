use crate::model::{Node, Protocol};
use crate::support::decode::b64_decode_text;

use super::super::support::{optional_string, required_string};

pub fn parse_vmess(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let payload = line.trim_start_matches("vmess://");
    let data: serde_json::Value = serde_json::from_str(&b64_decode_text(payload)?)?;

    let address = required_string(&data, "add")?;
    let port: u16 = required_string(&data, "port")?.parse()?;

    Ok(Node {
        protocol: Protocol::Vmess,
        address,
        port,
        username: None,
        uuid: optional_string(&data, "id"),
        password: None,
        method: None,
        network: optional_string(&data, "net").unwrap_or_else(|| "tcp".to_string()),
        tls: optional_string(&data, "tls"),
        sni: optional_string(&data, "sni"),
        host: optional_string(&data, "host"),
        path: optional_string(&data, "path"),
        name: optional_string(&data, "ps"),
        raw_config: line.to_string(),
    })
}
