use thiserror::Error;

use crate::config::{ConfigParseError, parse_link};
use crate::model::{Node, Protocol};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EngineMode {
    Auto,
    Xray,
    SingBox,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResolvedEngine {
    Xray,
    SingBox,
}

impl std::fmt::Display for ResolvedEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xray => formatter.write_str("xray"),
            Self::SingBox => formatter.write_str("sing-box"),
        }
    }
}

#[derive(Debug)]
pub struct ParsedEntry {
    pub node: Node,
    pub engine: ResolvedEngine,
}

#[derive(Debug, Error)]
pub enum ParseServiceError {
    #[error("parse failed for input #{index} ({context}): {message}")]
    Indexed {
        index: usize,
        context: String,
        message: String,
    },
}

pub fn parse_single(link: &str, mode: EngineMode) -> Result<Option<ParsedEntry>, ConfigParseError> {
    let node = match parse_link(link)? {
        Some(node) => node,
        None => return Ok(None),
    };
    let engine = resolve_engine(mode, node.protocol.clone())?;
    Ok(Some(ParsedEntry { node, engine }))
}

pub fn parse_batch(
    inputs: &[(String, String)],
    mode: EngineMode,
) -> Result<Vec<ParsedEntry>, ParseServiceError> {
    let mut parsed = Vec::with_capacity(inputs.len());

    for (index, (source, link)) in inputs.iter().enumerate() {
        let entry = parse_single(link, mode).map_err(|error| ParseServiceError::Indexed {
            index: index + 1,
            context: source.clone(),
            message: error.to_string(),
        })?;
        if let Some(entry) = entry {
            parsed.push(entry);
        }
    }

    Ok(parsed)
}

pub fn resolve_engine(mode: EngineMode, protocol: Protocol) -> Result<ResolvedEngine, ConfigParseError> {
    match mode {
        EngineMode::Auto => {
            if matches!(protocol, Protocol::Hy2) {
                Ok(ResolvedEngine::SingBox)
            } else {
                Ok(ResolvedEngine::Xray)
            }
        }
        EngineMode::Xray => {
            if matches!(protocol, Protocol::Hy2) {
                return Err(ConfigParseError::UnsupportedScheme(
                    "hysteria2/hy2 is not compatible with xray engine".to_string(),
                ));
            }
            Ok(ResolvedEngine::Xray)
        }
        EngineMode::SingBox => Ok(ResolvedEngine::SingBox),
    }
}

#[cfg(test)]
mod tests {
    use super::{EngineMode, ResolvedEngine, parse_batch, parse_single};

    #[test]
    fn resolves_auto_engine_for_hy2_link() {
        let parsed = parse_single("hy2://secret@example.com:443#n", EngineMode::Auto)
            .expect("hy2 parse should pass")
            .expect("entry should exist");
        assert!(matches!(parsed.engine, ResolvedEngine::SingBox));
    }

    #[test]
    fn parse_batch_returns_indexed_error_context() {
        let inputs = vec![
            ("stdin line 1".to_string(), "not-a-url".to_string()),
            (
                "stdin line 2".to_string(),
                "vless://uuid-123@example.com:443?type=tcp#ok".to_string(),
            ),
        ];
        let error = parse_batch(&inputs, EngineMode::Auto).expect_err("must fail first bad line");
        let rendered = error.to_string();
        assert!(rendered.contains("input #1"));
        assert!(rendered.contains("stdin line 1"));
        assert!(!rendered.contains("stdin line 2"));
    }
}
