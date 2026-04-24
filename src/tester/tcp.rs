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
    let target = format!("{}:{}", address, port);
    let start = Instant::now();

    match timeout(timeout_duration, TcpStream::connect(&target)).await {
        Ok(Ok(_stream)) => {
            let elapsed = start.elapsed();
            TcpResult {
                success: true,
                latency_ms: Some(elapsed.as_millis() as u32),
                failure_kind: None,
                failure_reason: None,
            }
        }
        Ok(Err(e)) => {
            let (kind, reason) = classify_tcp_error(&e);
            TcpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            }
        }
        Err(_) => TcpResult {
            success: false,
            latency_ms: None,
            failure_kind: Some(FailureKind::Timeout),
            failure_reason: Some(format!("Connection timeout after {:?}", timeout_duration)),
        },
    }
}

fn classify_tcp_error(error: &std::io::Error) -> (FailureKind, String) {
    use std::io::ErrorKind;

    match error.kind() {
        ErrorKind::ConnectionRefused => (FailureKind::Refused, "Connection refused".to_string()),
        ErrorKind::TimedOut => (FailureKind::Timeout, "Connection timed out".to_string()),
        ErrorKind::NotFound => (FailureKind::Dns, "DNS resolution failed".to_string()),
        ErrorKind::PermissionDenied => (
            FailureKind::PermissionDenied,
            "Permission denied".to_string(),
        ),
        _ => {
            // Check error message for more clues
            let msg = error.to_string().to_lowercase();
            if msg.contains("unreachable") || msg.contains("no route") {
                (FailureKind::Unreachable, "Network unreachable".to_string())
            } else if msg.contains("dns") || msg.contains("resolve") {
                (FailureKind::Dns, format!("DNS error: {}", error))
            } else {
                (FailureKind::Unknown, format!("Connection error: {}", error))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcp_check_success() {
        // Test against a known good service (Google DNS)
        let result = tcp_check("8.8.8.8", 53, Duration::from_secs(5)).await;
        assert!(result.success);
        assert!(result.latency_ms.is_some());
    }

    #[tokio::test]
    async fn test_tcp_check_refused() {
        // Test against localhost on a port that's likely not listening
        let result = tcp_check("127.0.0.1", 9999, Duration::from_secs(2)).await;
        assert!(!result.success);
        assert!(matches!(
            result.failure_kind,
            Some(FailureKind::Refused) | Some(FailureKind::Timeout)
        ));
    }

    #[tokio::test]
    async fn test_tcp_check_timeout() {
        // Test against a non-routable IP (should timeout)
        let result = tcp_check("192.0.2.1", 80, Duration::from_secs(1)).await;
        assert!(!result.success);
        assert!(matches!(
            result.failure_kind,
            Some(FailureKind::Timeout) | Some(FailureKind::Unreachable)
        ));
    }

    #[tokio::test]
    async fn test_tcp_check_dns_failure() {
        let result = tcp_check(
            "invalid.host.that.does.not.exist.example",
            80,
            Duration::from_secs(2),
        )
        .await;
        assert!(!result.success);
        assert!(matches!(result.failure_kind, Some(FailureKind::Dns)));
    }
}
