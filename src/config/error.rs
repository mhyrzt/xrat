use thiserror::Error;

use crate::support::decode::DecodeError;

#[derive(Debug, Error)]
pub enum ConfigParseError {
    #[error("invalid URL")]
    Url(#[from] url::ParseError),
    #[error("invalid JSON")]
    Json(#[from] serde_json::Error),
    #[error("invalid base64 payload")]
    Decode(#[from] DecodeError),
    #[error("invalid number")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("missing address or port")]
    MissingAddressOrPort,
    #[error("missing base64 userinfo")]
    MissingBase64Userinfo,
    #[error("invalid Shadowsocks userinfo format")]
    InvalidShadowsocksUserinfo,
    #[error("missing required {key} field in {context}")]
    MissingRequiredField {
        context: &'static str,
        key: &'static str,
    },
}
