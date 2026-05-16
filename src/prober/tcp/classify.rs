use super::FailureKind;

pub(super) fn classify_tcp_error(error: &std::io::Error) -> (FailureKind, String) {
    use std::io::ErrorKind;

    match error.kind() {
        ErrorKind::ConnectionRefused => (FailureKind::Refused, "Connection refused".to_string()),
        ErrorKind::TimedOut => (FailureKind::Timeout, "Connection timed out".to_string()),
        ErrorKind::AddrNotAvailable | ErrorKind::NotFound => {
            (FailureKind::Dns, "DNS resolution failed".to_string())
        }
        ErrorKind::PermissionDenied => (
            FailureKind::PermissionDenied,
            "Permission denied".to_string(),
        ),
        _ => {
            let msg = error.to_string().to_lowercase();
            if msg.contains("unreachable") || msg.contains("no route") {
                (FailureKind::Unreachable, "Network unreachable".to_string())
            } else if msg.contains("dns")
                || msg.contains("resolve")
                || msg.contains("name or service not known")
                || msg.contains("failed to lookup address information")
            {
                (FailureKind::Dns, format!("DNS error: {error}"))
            } else {
                (FailureKind::Unknown, format!("Connection error: {error}"))
            }
        }
    }
}

pub(super) fn classify_dns_error(error: &std::io::Error) -> (FailureKind, String) {
    let message = error.to_string().to_lowercase();
    if message.contains("temporary failure") || message.contains("timed out") {
        (
            FailureKind::Timeout,
            format!("DNS lookup timed out: {error}"),
        )
    } else {
        (FailureKind::Dns, format!("DNS resolution failed: {error}"))
    }
}
