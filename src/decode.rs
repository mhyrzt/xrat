use base64::Engine;
use serde_json::Value;

pub fn b64_decode_text(data: &str) -> Result<String, Box<dyn std::error::Error>> {
    let padded = with_padding(data);
    let decoded = base64::engine::general_purpose::URL_SAFE
        .decode(padded.as_bytes())
        .or_else(|_| {
            base64::engine::general_purpose::STANDARD.decode(padded.as_bytes())
        })?;
    Ok(String::from_utf8_lossy(&decoded).into_owned())
}

fn decode_b64_bytes(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let padded = with_padding(std::str::from_utf8(data)?);
    let decoded = base64::engine::general_purpose::STANDARD.decode(padded.as_bytes())?;
    Ok(String::from_utf8_lossy(&decoded).trim().to_string())
}

pub fn decode_or_json_text(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(decoded_text) = decode_b64_bytes(data) {
        if !decoded_text.is_empty() {
            return Ok(decoded_text);
        }
    }

    let raw_text = String::from_utf8_lossy(data).trim().to_string();
    if raw_text.is_empty() {
        return Err("response body is empty".into());
    }

    serde_json::from_str::<Value>(&raw_text)
        .map_err(|_| "response is neither valid base64 nor valid JSON")?;

    Ok(raw_text)
}

pub fn decode_or_raw_text(data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(decoded_text) = decode_b64_bytes(data) {
        if !decoded_text.is_empty() {
            return Ok(decoded_text);
        }
    }

    let raw_text = String::from_utf8_lossy(data).trim().to_string();
    if raw_text.is_empty() {
        return Err("response body is empty".into());
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
