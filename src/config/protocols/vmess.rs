use crate::config::ConfigParseError;
use crate::model::{Node, Protocol};
use crate::support::decode::b64_decode_text;

use super::super::parsing_helpers::{optional_string, required_string};

pub fn parse_vmess(line: &str) -> Result<Node, ConfigParseError> {
    let payload = line.trim_start_matches("vmess://");
    let data: serde_json::Value = serde_json::from_str(&b64_decode_text(payload)?)?;

    let address = required_string(&data, "add")?;
    let port: u16 = required_string(&data, "port")?.parse()?;
    let extensions = data.as_object().and_then(|object| {
        let mut extensions = object
            .clone()
            .into_iter()
            .collect::<std::collections::BTreeMap<_, _>>();
        for key in [
            "add", "port", "id", "net", "tls", "sni", "host", "path", "ps",
        ] {
            extensions.remove(key);
        }
        (!extensions.is_empty()).then_some(extensions)
    });

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
        extensions,
        raw_config: line.to_string(),
    })
}
