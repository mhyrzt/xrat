use url::Url;

use crate::config::ConfigParseError;
use crate::model::{Node, Protocol};

use super::super::parsing_helpers::{
    empty_to_none, parse_query_pairs, percent_decode, query_extensions, username_or_none,
};

pub fn parse_trojan(line: &str) -> Result<Node, ConfigParseError> {
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

    Ok(Node {
        protocol: Protocol::Trojan,
        address,
        port,
        username: None,
        uuid: None,
        password: username_or_none(&parsed),
        method: None,
        network: query
            .get("type")
            .cloned()
            .unwrap_or_else(|| "tcp".to_string()),
        tls: query
            .get("security")
            .cloned()
            .or_else(|| Some("tls".to_string())),
        sni: query.get("sni").cloned(),
        host: query.get("host").cloned(),
        path: empty_to_none(percent_decode(path)),
        name: fragment.and_then(empty_to_none),
        extensions: query_extensions(
            parsed.query().unwrap_or_default(),
            &["type", "security", "sni", "host", "path"],
        ),
        raw_config: line.to_string(),
    })
}
