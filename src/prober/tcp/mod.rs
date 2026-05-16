mod check;
mod classify;
mod model;

use super::FailureKind;
pub use check::tcp_check;
pub use model::TcpResult;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};
    use std::time::Duration;

    #[test]
    fn classifies_connection_refused_errors() {
        let (kind, reason) =
            classify::classify_tcp_error(&Error::new(ErrorKind::ConnectionRefused, "refused"));
        assert_eq!(kind, FailureKind::Refused);
        assert_eq!(reason, "Connection refused");
    }

    #[test]
    fn classifies_timeout_errors() {
        let (kind, reason) =
            classify::classify_tcp_error(&Error::new(ErrorKind::TimedOut, "timed out"));
        assert_eq!(kind, FailureKind::Timeout);
        assert_eq!(reason, "Connection timed out");
    }

    #[test]
    fn classifies_permission_errors() {
        let (kind, reason) =
            classify::classify_tcp_error(&Error::new(ErrorKind::PermissionDenied, "blocked"));
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
