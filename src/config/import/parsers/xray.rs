use super::super::{ImportParseError, ImportResult};
use crate::xray::parsing::XrayConfig;

pub fn parse_xray_json(input: &str) -> Result<ImportResult, ImportParseError> {
    let _config: XrayConfig = XrayConfig::from_json_loose(input)?;
    Ok(ImportResult {
        nodes: vec![],
        errors: vec![],
        metadata: None,
    })
}
