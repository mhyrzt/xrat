use super::super::{ImportParseError, ImportResult};
use super::plain::parse_plain_list;

pub fn parse_single_link(input: &str) -> Result<ImportResult, ImportParseError> {
    let node = crate::config::line::parse_line(input).ok_or(ImportParseError::InvalidShareLink)?;
    Ok(ImportResult {
        nodes: vec![node],
        errors: vec![],
        metadata: None,
    })
}

pub fn parse_base64_subscription(input: &str) -> Result<ImportResult, ImportParseError> {
    use crate::support::decode::b64_decode_text;

    let decoded = b64_decode_text(input.trim())?;
    parse_plain_list(&decoded)
}
