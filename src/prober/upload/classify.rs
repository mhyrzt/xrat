use super::FailureKind;

pub fn classify_request_error(error: &reqwest::Error) -> (FailureKind, String) {
    if error.is_timeout() {
        (FailureKind::Timeout, "Upload request timeout".to_string())
    } else if error.to_string().to_lowercase().contains("tls") {
        (FailureKind::Tls, format!("TLS handshake failed: {error}"))
    } else if error.to_string().to_lowercase().contains("407") {
        (
            FailureKind::Auth,
            format!("Proxy authentication failed: {error}"),
        )
    } else if error.is_connect() {
        (
            FailureKind::Proxy,
            format!("Proxy connection failed: {error}"),
        )
    } else if error.is_request() {
        (
            FailureKind::Proxy,
            format!("Upload request failed: {error}"),
        )
    } else {
        (FailureKind::Unknown, format!("Upload HTTP error: {error}"))
    }
}
