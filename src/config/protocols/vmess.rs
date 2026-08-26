use crate::config::ConfigParseError;
use crate::model::{Node, Protocol};
use crate::support::decode::b64_decode_text;

use super::super::parsing_helpers::{
    empty_to_none, optional_string, parse_query_pairs, percent_decode, query_extensions,
    reject_duplicate_query_parameters, required_string, username_or_none,
};

pub fn parse_vmess(line: &str) -> Result<Node, ConfigParseError> {
    let payload = line.trim_start_matches("vmess://");
    if payload.contains('@') {
        return parse_vmess_url(line);
    }
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

fn parse_vmess_url(line: &str) -> Result<Node, ConfigParseError> {
    let parsed = url::Url::parse(line)?;
    reject_duplicate_query_parameters(parsed.query().unwrap_or_default())?;
    let address = parsed
        .host_str()
        .ok_or(ConfigParseError::MissingAddressOrPort)?
        .to_string();
    let port = parsed
        .port()
        .ok_or(ConfigParseError::MissingAddressOrPort)?;
    let uuid = username_or_none(&parsed).ok_or(ConfigParseError::MissingRequiredField {
        context: "VMess URL",
        key: "id",
    })?;
    uuid::Uuid::parse_str(&uuid).map_err(|error| ConfigParseError::InvalidField {
        field: "VMess id",
        message: error.to_string(),
    })?;

    let query_text = parsed.query().unwrap_or_default();
    let query = parse_query_pairs(query_text);
    let path = query.get("path").map(String::as_str).unwrap_or_default();
    let extensions = query_extensions(query_text, &["type", "security", "sni", "host", "path"]);

    Ok(Node {
        protocol: Protocol::Vmess,
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
        name: parsed
            .fragment()
            .map(percent_decode)
            .and_then(empty_to_none),
        extensions,
        raw_config: line.to_string(),
    })
}
