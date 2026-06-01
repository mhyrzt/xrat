use serde_json::Value;

use super::DecodeError;
use super::b64::decode_b64_bytes;

pub fn decode_or_json_text(data: &[u8]) -> Result<String, DecodeError> {
    if let Ok(decoded_text) = decode_b64_bytes(data)
        && !decoded_text.is_empty()
    {
        return Ok(decoded_text);
    }

    let raw_text = String::from_utf8_lossy(data).trim().to_string();
    if raw_text.is_empty() {
        return Err(DecodeError::EmptyBody);
    }

    serde_json::from_str::<Value>(&raw_text).map_err(|_| DecodeError::NotBase64OrJson)?;

    Ok(raw_text)
}

pub fn decode_or_raw_text(data: &[u8]) -> Result<String, DecodeError> {
    if let Ok(decoded_text) = decode_b64_bytes(data)
        && !decoded_text.is_empty()
    {
        return Ok(decoded_text);
    }

    let raw_text = String::from_utf8_lossy(data).trim().to_string();
    if raw_text.is_empty() {
        return Err(DecodeError::EmptyBody);
    }

    Ok(raw_text)
}
