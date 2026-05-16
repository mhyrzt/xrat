use crate::prober::FailureKind;
use crate::xray::XrayProcessError;

pub(super) fn classify_xray_error(error: &XrayProcessError) -> (FailureKind, String) {
    match error {
        XrayProcessError::SpawnError(_) => (
            FailureKind::Process,
            format!("Failed to spawn xray: {error}"),
        ),
        XrayProcessError::StartupTimeout => {
            (FailureKind::Timeout, "Xray startup timeout".to_string())
        }
        XrayProcessError::ProcessExited(_) => (
            FailureKind::Process,
            "Xray process exited unexpectedly".to_string(),
        ),
        XrayProcessError::PortNotReady(_) => (
            FailureKind::Process,
            format!("Xray port not ready: {error}"),
        ),
        _ => (FailureKind::Process, format!("Xray error: {error}")),
    }
}

pub(super) fn classify_request_error(error: &reqwest::Error) -> (FailureKind, String) {
    if error.is_timeout() {
        (FailureKind::Timeout, "Download request timeout".to_string())
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
            format!("Download request failed: {error}"),
        )
    } else {
        (
            FailureKind::Unknown,
            format!("Download HTTP error: {error}"),
        )
    }
}
