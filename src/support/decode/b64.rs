use base64::Engine;

use super::DecodeError;

pub fn b64_decode_text(data: &str) -> Result<String, DecodeError> {
    let padded = with_padding(data);
    let decoded = base64::engine::general_purpose::URL_SAFE
        .decode(padded.as_bytes())
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(padded.as_bytes()))?;
    Ok(String::from_utf8_lossy(&decoded).into_owned())
}

pub(super) fn decode_b64_bytes(data: &[u8]) -> Result<String, DecodeError> {
    let padded = with_padding(std::str::from_utf8(data)?);
    let decoded = base64::engine::general_purpose::STANDARD.decode(padded.as_bytes())?;
    Ok(String::from_utf8_lossy(&decoded).trim().to_string())
}

pub(super) fn with_padding(data: &str) -> String {
    let mut padded = data.trim().to_string();
    let remainder = padded.len() % 4;
    if remainder != 0 {
        padded.push_str(&"=".repeat(4 - remainder));
    }

    padded
}
