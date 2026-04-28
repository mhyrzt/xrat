use base64::Engine;
use serde_json::Value;
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

pub fn b64_decode_text(data: &str) -> Result<String, DecodeError> {
    let padded = with_padding(data);
    let decoded = base64::engine::general_purpose::URL_SAFE
        .decode(padded.as_bytes())
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(padded.as_bytes()))?;
    Ok(String::from_utf8_lossy(&decoded).into_owned())
}

fn decode_b64_bytes(data: &[u8]) -> Result<String, DecodeError> {
    let padded = with_padding(std::str::from_utf8(data)?);
    let decoded = base64::engine::general_purpose::STANDARD.decode(padded.as_bytes())?;
    Ok(String::from_utf8_lossy(&decoded).trim().to_string())
}

pub fn decode_or_json_text(data: &[u8]) -> Result<String, DecodeError> {
    if let Ok(decoded_text) = decode_b64_bytes(data) {
        if !decoded_text.is_empty() {
            return Ok(decoded_text);
        }
    }

    let raw_text = String::from_utf8_lossy(data).trim().to_string();
    if raw_text.is_empty() {
        return Err(DecodeError::EmptyBody);
    }

    serde_json::from_str::<Value>(&raw_text).map_err(|_| DecodeError::NotBase64OrJson)?;

    Ok(raw_text)
}

pub fn decode_or_raw_text(data: &[u8]) -> Result<String, DecodeError> {
    if let Ok(decoded_text) = decode_b64_bytes(data) {
        if !decoded_text.is_empty() {
            return Ok(decoded_text);
        }
    }

    let raw_text = String::from_utf8_lossy(data).trim().to_string();
    if raw_text.is_empty() {
        return Err(DecodeError::EmptyBody);
    }

    Ok(raw_text)
}

fn with_padding(data: &str) -> String {
    let mut padded = data.trim().to_string();
    let remainder = padded.len() % 4;
    if remainder != 0 {
        padded.push_str(&"=".repeat(4 - remainder));
    }

    padded
}

#[cfg(test)]
mod tests {
    use super::{DecodeError, b64_decode_text, decode_or_json_text, decode_or_raw_text};

    #[test]
    fn decodes_url_safe_base64_text() {
        let decoded = b64_decode_text("aGVsbG8td29ybGQ").expect("base64 should decode");

        assert_eq!(decoded, "hello-world");
    }

    #[test]
    fn falls_back_to_raw_text_when_not_base64() {
        let decoded = decode_or_raw_text(b"vless://example").expect("raw text should decode");

        assert_eq!(decoded, "vless://example");
    }

    #[test]
    fn accepts_raw_json_when_not_base64() {
        let decoded = decode_or_json_text(br#"{"outbounds":[]}"#).expect("json should decode");

        assert_eq!(decoded, r#"{"outbounds":[]}"#);
    }

    #[test]
    fn rejects_empty_body() {
        let error = decode_or_raw_text(b"   ").expect_err("empty body should fail");

        assert!(matches!(error, DecodeError::EmptyBody));
    }

    #[test]
    fn rejects_non_json_when_json_required() {
        let error = decode_or_json_text(b"not json").expect_err("invalid json should fail");

        assert!(matches!(error, DecodeError::NotBase64OrJson));
    }
}
