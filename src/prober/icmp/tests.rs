use super::*;

#[tokio::test]
async fn test_icmp_ping_localhost() {
    let result = icmp_ping("127.0.0.1", Duration::from_secs(2)).await;
    if matches!(result.failure_kind, Some(FailureKind::PermissionDenied)) {
        return;
    }

    assert!(result.success);
    assert!(result.latency_ms.is_some());
}

#[tokio::test]
async fn test_icmp_ping_invalid_host() {
    let result = icmp_ping(
        "invalid.host.that.does.not.exist.example",
        Duration::from_secs(2),
    )
    .await;
    assert!(!result.success);
    assert!(matches!(result.failure_kind, Some(FailureKind::Dns)));
}

#[test]
fn test_parse_ping_latency() {
    let output = "64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=0.123 ms";
    assert_eq!(parse_ping_latency(output), Some(0));

    let output = "64 bytes from 127.0.0.1: icmp_seq=1 ttl=64 time=12.5 ms";
    assert_eq!(parse_ping_latency(output), Some(13));
}

#[test]
fn classifies_permission_denied_output() {
    let (kind, reason) = classify_ping_failure("ping: socket: Operation not permitted");
    assert_eq!(kind, FailureKind::PermissionDenied);
    assert_eq!(reason, "ICMP ping requires additional permissions");
}
