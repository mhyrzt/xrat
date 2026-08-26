use url::Url;

use crate::config::ConfigParseError;
use crate::model::{Node, Protocol};

use super::super::parsing_helpers::{
    empty_to_none, parse_query_pairs, percent_decode, query_extensions,
    reject_duplicate_query_parameters, username_or_none,
};

pub fn parse_vless(line: &str) -> Result<Node, ConfigParseError> {
    let parsed = Url::parse(line)?;
    reject_duplicate_query_parameters(parsed.query().unwrap_or_default())?;
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

    let extensions = query_extensions(
        parsed.query().unwrap_or_default(),
        &["type", "security", "sni", "host", "path"],
    );

    let uuid = username_or_none(&parsed).ok_or(ConfigParseError::MissingRequiredField {
        context: "VLESS URL",
        key: "id",
    })?;

    Ok(Node {
        protocol: Protocol::Vless,
        address,
        port,
        username: None,
        uuid: Some(uuid),
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
        extensions,
        raw_config: line.to_string(),
    })
}
