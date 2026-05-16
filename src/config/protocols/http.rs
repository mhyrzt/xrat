use url::Url;

use crate::config::ConfigParseError;
use crate::model::{Node, Protocol};

use super::super::parsing_helpers::{
    empty_to_none, password_or_none, percent_decode, username_or_none,
};

pub fn parse_http(line: &str) -> Result<Node, ConfigParseError> {
    let parsed = Url::parse(line)?;
    let address = parsed
        .host_str()
        .ok_or(ConfigParseError::MissingAddressOrPort)?
        .to_string();
    let port = parsed
        .port_or_known_default()
        .ok_or(ConfigParseError::MissingAddressOrPort)?;

    Ok(Node {
        protocol: Protocol::Http,
        address,
        port,
        username: username_or_none(&parsed),
        uuid: None,
        password: password_or_none(&parsed),
        method: None,
        network: "tcp".to_string(),
        tls: (parsed.scheme() == "https").then(|| "tls".to_string()),
        sni: None,
        host: None,
        path: None,
        name: parsed
            .fragment()
            .map(percent_decode)
            .and_then(empty_to_none),
        extensions: None,
        raw_config: line.to_string(),
    })
}
