use thiserror::Error;

/// A complete Xray configuration retained exactly as supplied.
#[derive(Clone, PartialEq, Eq)]
pub struct XrayDocument {
    bytes: Vec<u8>,
}

impl std::fmt::Debug for XrayDocument {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("XrayDocument([REDACTED])")
    }
}

#[derive(Debug, Error)]
pub enum XrayDocumentError {
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("an Xray configuration must be a JSON object")]
    NotObject,
}

impl XrayDocument {
    /// Checks JSON syntax and the root type; Xray itself validates the schema.
    pub fn parse(bytes: &[u8]) -> Result<Self, XrayDocumentError> {
        let value: serde_json::Value = serde_json::from_slice(bytes)?;
        if !value.is_object() {
            return Err(XrayDocumentError::NotObject);
        }
        Ok(Self {
            bytes: bytes.to_vec(),
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_full_configuration_and_unknown_fields_verbatim() {
        let input = br#"{
  "dns": {"servers": ["1.1.1.1"]},
  "routing": {"rules": [{"outboundTag": "dns-out"}]},
  "outbounds": [{"protocol": "vless", "streamSettings": {"network": "xhttp", "xhttpSettings": {"extra": {"xmux": {"maxConcurrency": 4}}}}}, {"tag": "dns-out", "protocol": "dns"}],
  "futureField": {"unrecognized": true}
}
"#;
        assert_eq!(XrayDocument::parse(input).unwrap().as_bytes(), input);
    }

    #[test]
    fn rejects_invalid_json_and_non_object_roots() {
        assert!(matches!(
            XrayDocument::parse(b"{"),
            Err(XrayDocumentError::InvalidJson(_))
        ));
        assert!(matches!(
            XrayDocument::parse(b"[]"),
            Err(XrayDocumentError::NotObject)
        ));
        assert!(matches!(
            XrayDocument::parse(b"null"),
            Err(XrayDocumentError::NotObject)
        ));
    }

    #[test]
    fn debug_does_not_expose_credentials() {
        let document = XrayDocument::parse(br#"{"password":"sensitive-value"}"#).unwrap();
        assert!(!format!("{document:?}").contains("sensitive-value"));
    }
}
