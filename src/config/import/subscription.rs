use thiserror::Error;

use super::{ImportParseError, ImportResult, SubscriptionMetadata};
use crate::support::decode::DecodeError;

#[derive(Debug, Error)]
pub enum SubscriptionFetchError {
    #[error("invalid subscription URL")]
    Decode(#[from] DecodeError),
    #[error("request failed")]
    Request(#[from] reqwest::Error),
    #[error("subscription content could not be parsed")]
    Parse(#[from] ImportParseError),
}

pub async fn fetch_subscription(url: &str) -> Result<ImportResult, SubscriptionFetchError> {
    let url = normalize_subscription_url(url)?;
    let response = reqwest::get(&url).await?;

    let mut metadata = SubscriptionMetadata {
        upload: None,
        download: None,
        total: None,
        expire: None,
        status: None,
    };

    if let Some(userinfo) = response.headers().get("subscription-userinfo") {
        if let Ok(userinfo_str) = userinfo.to_str() {
            parse_subscription_userinfo(userinfo_str, &mut metadata);
        }
    }

    let body = response.text().await?;
    let body = body.trim_start_matches('\u{feff}');

    let decoded = match crate::support::decode::b64_decode_text(body) {
        Ok(decoded) => decoded,
        Err(_) => body.to_string(),
    };

    let mut result = super::parsers::parse_plain_list(&decoded)?;

    if metadata.upload.is_some()
        || metadata.download.is_some()
        || metadata.total.is_some()
        || metadata.expire.is_some()
    {
        result.metadata = Some(metadata);
    }

    Ok(result)
}

fn normalize_subscription_url(url: &str) -> Result<String, DecodeError> {
    let url = url.trim();

    if url.starts_with("sub://") {
        let encoded = url.trim_start_matches("sub://");
        let decoded = crate::support::decode::b64_decode_text(encoded)?;
        return Ok(decoded);
    }

    if !url.contains("://") {
        return Ok(format!("http://{}", url));
    }

    Ok(url.to_string())
}

fn parse_subscription_userinfo(userinfo: &str, metadata: &mut SubscriptionMetadata) {
    for part in userinfo.split(';') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "upload" => metadata.upload = value.parse().ok(),
                "download" => metadata.download = value.parse().ok(),
                "total" => metadata.total = value.parse().ok(),
                "expire" => metadata.expire = value.parse().ok(),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_sub_protocol() {
        let encoded = "aHR0cDovL2V4YW1wbGUuY29tL3N1Yg==";
        let url = format!("sub://{}", encoded);
        let normalized = normalize_subscription_url(&url).unwrap();
        assert_eq!(normalized, "http://example.com/sub");
    }

    #[test]
    fn test_normalize_bare_domain() {
        let url = "example.com/subscription";
        let normalized = normalize_subscription_url(url).unwrap();
        assert_eq!(normalized, "http://example.com/subscription");
    }

    #[test]
    fn test_normalize_full_url() {
        let url = "https://example.com/subscription";
        let normalized = normalize_subscription_url(url).unwrap();
        assert_eq!(normalized, "https://example.com/subscription");
    }

    #[test]
    fn test_parse_userinfo() {
        let userinfo = "upload=1024; download=2048; total=10240; expire=1234567890";
        let mut metadata = SubscriptionMetadata {
            upload: None,
            download: None,
            total: None,
            expire: None,
            status: None,
        };

        parse_subscription_userinfo(userinfo, &mut metadata);

        assert_eq!(metadata.upload, Some(1024));
        assert_eq!(metadata.download, Some(2048));
        assert_eq!(metadata.total, Some(10240));
        assert_eq!(metadata.expire, Some(1234567890));
    }
}
