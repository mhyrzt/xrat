mod b64;
mod text;

#[cfg(test)]
mod tests;

pub use b64::b64_decode_text;
pub use text::{decode_or_json_text, decode_or_raw_text};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("invalid base64 text")]
    Base64(#[from] base64::DecodeError),
    #[error("input is not valid UTF-8")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("response body is empty")]
    EmptyBody,
    #[error("response is neither valid base64 nor valid JSON")]
    NotBase64OrJson,
}
