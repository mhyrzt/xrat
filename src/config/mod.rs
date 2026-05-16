mod error;
mod import;
mod line;
mod normalize;
mod parse_service;
mod protocols;
mod support;

use std::collections::HashSet;

use crate::model::Node;

pub use error::ConfigParseError;
pub use import::{ImportMode, ImportResult, SubscriptionMetadata, parse_import};
pub use parse_service::{
    EngineMode, ParseServiceError, ParsedEntry, ResolvedEngine, parse_batch as parse_links_batch,
    parse_single as parse_link_with_engine,
};

pub fn parse_text(config_text: &str) -> Vec<Node> {
    let mut nodes = Vec::new();
    let mut seen = HashSet::new();

    for line in config_text.lines() {
        let Some(mut node) = line::parse_line(line) else {
            continue;
        };

        normalize::normalize(&mut node);
        if seen.insert(node.dedup_key()) {
            nodes.push(node);
        }
    }

    nodes
}

pub fn parse_link(link: &str) -> Result<Option<Node>, ConfigParseError> {
    let line = link.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }

    let mut node = match line {
        value if value.starts_with("vless://") => protocols::parse_vless(value)?,
        value if value.starts_with("vmess://") => protocols::parse_vmess(value)?,
        value if value.starts_with("ss://") => protocols::parse_ss(value)?,
        value if value.starts_with("trojan://") => protocols::parse_trojan(value)?,
        value if value.starts_with("http://") || value.starts_with("https://") => {
            protocols::parse_http(value)?
        }
        value if value.starts_with("socks5://") => protocols::parse_socks5(value)?,
        value if value.starts_with("hysteria2://") || value.starts_with("hy2://") => {
            protocols::parse_hy2(value)?
        }
        _ => return Err(ConfigParseError::UnsupportedScheme(line.to_string())),
    };

    normalize::normalize(&mut node);
    Ok(Some(node))
}

#[cfg(test)]
mod tests;
