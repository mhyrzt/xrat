use thiserror::Error;

use crate::config::ConfigParseError;
use crate::config::xray::XrayConfigError;
use crate::support::decode::DecodeError;

#[derive(Debug, Error)]
pub enum ImportParseError {
    #[error("failed to parse share link")]
    InvalidShareLink,
    #[error("invalid base64 subscription")]
    Decode(#[from] DecodeError),
    #[error("invalid SIP008 JSON")]
    Json(#[from] serde_json::Error),
    #[error("SIP008 JSON must have 'servers' array")]
    MissingSip008Servers,
    #[error("missing '{0}' field")]
    MissingSip008Field(&'static str),
    #[error("invalid Xray JSON")]
    Xray(#[from] XrayConfigError),
    #[error("invalid config node")]
    Config(#[from] ConfigParseError),
}
