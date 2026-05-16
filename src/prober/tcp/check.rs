use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::time::timeout;

use super::FailureKind;
use super::classify::{classify_dns_error, classify_tcp_error};
use super::model::TcpResult;

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
