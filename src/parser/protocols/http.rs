use url::Url;

use crate::model::{Node, Protocol};

use super::super::support::{empty_to_none, password_or_none, percent_decode, username_or_none};

pub fn parse_http(line: &str) -> Result<Node, Box<dyn std::error::Error>> {
    let parsed = Url::parse(line)?;
    let address = parsed
        .host_str()
        .ok_or("missing address or port")?
        .to_string();
    let port = parsed
        .port_or_known_default()
        .ok_or("missing address or port")?;

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
    })
}
