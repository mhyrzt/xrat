use std::collections::BTreeMap;
use url::Url;

use crate::config::ConfigParseError;
use crate::model::{Node, Protocol};

use super::super::parsing_helpers::{
    empty_to_none, parse_query_pairs, percent_decode, username_or_none,
};

pub fn parse_vless(line: &str) -> Result<Node, ConfigParseError> {
    let parsed = Url::parse(line)?;
    let address = parsed
        .host_str()
        .ok_or(ConfigParseError::MissingAddressOrPort)?
        .to_string();
    let port = parsed
        .port()
        .ok_or(ConfigParseError::MissingAddressOrPort)?;
    let query = parse_query_pairs(parsed.query().unwrap_or_default());
    let fragment = parsed.fragment().map(percent_decode);
    let path = query.get("path").map(String::as_str).unwrap_or_default();

    let mut extensions = BTreeMap::new();
    for key in ["fp", "alpn", "mode", "flow", "pbk", "sid", "spx"] {
        if let Some(value) = query.get(key)
            && !value.is_empty()
        {
            extensions.insert(key.to_string(), value.clone());
        }
    }

    Ok(Node {
        protocol: Protocol::Vless,
        address,
        port,
        username: None,
        uuid: username_or_none(&parsed),
        password: None,
        method: None,
        network: query
            .get("type")
            .cloned()
            .unwrap_or_else(|| "tcp".to_string()),
        tls: query.get("security").cloned(),
        sni: query.get("sni").cloned(),
        host: query.get("host").cloned(),
        path: empty_to_none(percent_decode(path)),
        name: fragment.and_then(empty_to_none),
        extensions: if extensions.is_empty() {
            None
        } else {
            Some(extensions)
        },
        raw_config: line.to_string(),
    })
}
