use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::FailureKind;

#[derive(Debug, Clone)]
pub struct TcpResult {
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

/// Perform TCP connectivity check to the target address and port
pub async fn tcp_check(address: &str, port: u16, timeout_duration: Duration) -> TcpResult {
    let target = format!("{address}:{port}");
    let addresses = match tokio::net::lookup_host(&target).await {
        Ok(resolved) => resolved.collect::<Vec<_>>(),
        Err(error) => {
            let (kind, reason) = classify_dns_error(&error);
            return TcpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            };
        }
    };

    if addresses.is_empty() {
        return TcpResult {
            success: false,
            latency_ms: None,
            failure_kind: Some(FailureKind::Dns),
            failure_reason: Some("DNS resolution returned no addresses".to_string()),
        };
    }

    let start = Instant::now();
    let mut last_error = None;

    for target_addr in addresses {
        match timeout(timeout_duration, TcpStream::connect(target_addr)).await {
            Ok(Ok(_stream)) => {
                let elapsed = start.elapsed();
                return TcpResult {
                    success: true,
                    latency_ms: Some(elapsed.as_millis() as u32),
                    failure_kind: None,
                    failure_reason: None,
                };
            }
            Ok(Err(error)) => last_error = Some(error),
            Err(_) => {
                return TcpResult {
                    success: false,
                    latency_ms: None,
                    failure_kind: Some(FailureKind::Timeout),
                    failure_reason: Some(format!(
                        "Connection timeout after {:?}",
                        timeout_duration
                    )),
                };
            }
        }
    }

    let (kind, reason) = match last_error {
        Some(error) => classify_tcp_error(&error),
        None => (
            FailureKind::Unknown,
            "No addresses available for TCP probe".to_string(),
        ),
    };

    TcpResult {
        success: false,
        latency_ms: None,
        failure_kind: Some(kind),
        failure_reason: Some(reason),
    }
}

fn classify_tcp_error(error: &std::io::Error) -> (FailureKind, String) {
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
            // Check error message for more clues
            let msg = error.to_string().to_lowercase();
            if msg.contains("unreachable") || msg.contains("no route") {
                (FailureKind::Unreachable, "Network unreachable".to_string())
            } else if msg.contains("dns")
                || msg.contains("resolve")
                || msg.contains("name or service not known")
                || msg.contains("failed to lookup address information")
            {
                (FailureKind::Dns, format!("DNS error: {}", error))
            } else {
                (FailureKind::Unknown, format!("Connection error: {}", error))
            }
        }
    }
}

fn classify_dns_error(error: &std::io::Error) -> (FailureKind, String) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn classifies_connection_refused_errors() {
        let (kind, reason) =
            classify_tcp_error(&Error::new(ErrorKind::ConnectionRefused, "refused"));
        assert_eq!(kind, FailureKind::Refused);
        assert_eq!(reason, "Connection refused");
    }

    #[test]
    fn classifies_timeout_errors() {
        let (kind, reason) = classify_tcp_error(&Error::new(ErrorKind::TimedOut, "timed out"));
        assert_eq!(kind, FailureKind::Timeout);
        assert_eq!(reason, "Connection timed out");
    }

    #[test]
    fn classifies_permission_errors() {
        let (kind, reason) =
            classify_tcp_error(&Error::new(ErrorKind::PermissionDenied, "blocked"));
        assert_eq!(kind, FailureKind::PermissionDenied);
        assert_eq!(reason, "Permission denied");
    }

    #[tokio::test]
    async fn classifies_dns_lookup_failures() {
        let result = tcp_check("definitely-not-a-host.invalid", 80, Duration::from_secs(1)).await;
        assert!(!result.success);
        assert!(matches!(
            result.failure_kind,
            Some(FailureKind::Dns) | Some(FailureKind::Timeout)
        ));
    }
}
