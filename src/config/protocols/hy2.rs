use url::Url;

use crate::config::ConfigParseError;
use crate::model::{Node, Protocol};

use super::super::support::{empty_to_none, parse_query_pairs, percent_decode, username_or_none};

pub fn parse_hy2(line: &str) -> Result<Node, ConfigParseError> {
    let parsed = Url::parse(line)?;
    let address = parsed
        .host_str()
        .ok_or(ConfigParseError::MissingAddressOrPort)?
        .to_string();
    let port = parsed
        .port()
        .ok_or(ConfigParseError::MissingAddressOrPort)?;
    let query = parse_query_pairs(parsed.query().unwrap_or_default());

    Ok(Node {
        protocol: Protocol::Hy2,
        address,
        port,
        username: None,
        uuid: None,
        password: username_or_none(&parsed),
        method: None,
        network: "udp".to_string(),
        tls: Some("tls".to_string()),
        sni: query.get("sni").cloned(),
        host: query.get("obfs-password").cloned(),
        path: query.get("obfs").cloned(),
        name: parsed
            .fragment()
            .map(percent_decode)
            .and_then(empty_to_none),
        raw_config: line.to_string(),
    })
}
