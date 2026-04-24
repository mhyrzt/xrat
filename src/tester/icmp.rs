use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::process::Command;

use super::FailureKind;

#[derive(Debug, Clone)]
pub struct IcmpResult {
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

/// Perform ICMP ping check to the target address
pub async fn icmp_ping(address: &str, timeout: Duration) -> IcmpResult {
    // Try to resolve the address first
    let ip = match resolve_address(address).await {
        Ok(ip) => ip,
        Err(e) => {
            return IcmpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(FailureKind::Dns),
                failure_reason: Some(e),
            };
        }
    };

    // Use system ping command
    let result = ping_with_system_command(&ip.to_string(), timeout).await;

    result
}

async fn resolve_address(address: &str) -> Result<IpAddr, String> {
    // If already an IP, return it
    if let Ok(ip) = address.parse::<IpAddr>() {
        return Ok(ip);
    }

    // Resolve DNS
    tokio::net::lookup_host(format!("{}:0", address))
        .await
        .map_err(|e| format!("DNS resolution failed: {}", e))?
        .next()
        .map(|addr| addr.ip())
        .ok_or_else(|| "No IP address found".to_string())
}

async fn ping_with_system_command(ip: &str, timeout: Duration) -> IcmpResult {
    let timeout_secs = timeout.as_secs().max(1).to_string();

    // Detect OS and use appropriate ping command
    let (count_flag, timeout_flag) = if cfg!(target_os = "macos") {
        ("-c", "-t")
    } else if cfg!(target_os = "linux") {
        ("-c", "-W")
    } else if cfg!(target_os = "windows") {
        ("-n", "-w")
    } else {
        ("-c", "-W")
    };

    let start = Instant::now();
    let output = Command::new("ping")
        .arg(count_flag)
        .arg("1")
        .arg(timeout_flag)
        .arg(&timeout_secs)
        .arg(ip)
        .output()
        .await;

    let elapsed = start.elapsed();

    match output {
        Ok(output) => {
            if output.status.success() {
                // Parse latency from output
                let stdout = String::from_utf8_lossy(&output.stdout);
                let latency = parse_ping_latency(&stdout).unwrap_or(elapsed.as_millis() as u32);

                IcmpResult {
                    success: true,
                    latency_ms: Some(latency),
                    failure_kind: None,
                    failure_reason: None,
                }
            } else {
                // Check stderr for specific errors
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                let combined = format!("{}{}", stdout, stderr);

                let (kind, reason) = classify_ping_failure(&combined);

                IcmpResult {
                    success: false,
                    latency_ms: None,
                    failure_kind: Some(kind),
                    failure_reason: Some(reason),
                }
            }
        }
        Err(e) => {
            let kind = if e.kind() == std::io::ErrorKind::PermissionDenied {
                FailureKind::PermissionDenied
            } else {
                FailureKind::Unknown
            };

            IcmpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(kind),
                failure_reason: Some(format!("Failed to execute ping: {}", e)),
            }
        }
    }
}

fn parse_ping_latency(output: &str) -> Option<u32> {
    // Try to extract time= value from ping output
    // Example: "time=12.3 ms" or "time=12 ms"
    for line in output.lines() {
        if let Some(time_pos) = line.find("time=") {
            let time_str = &line[time_pos + 5..];
            if let Some(space_pos) = time_str.find(|c: char| c.is_whitespace()) {
                let num_str = &time_str[..space_pos];
                if let Ok(ms) = num_str.parse::<f64>() {
                    return Some(ms.round() as u32);
                }
            }
        }
    }
    None
}

fn classify_ping_failure(output: &str) -> (FailureKind, String) {
    let lower = output.to_lowercase();

    if lower.contains("unreachable") || lower.contains("no route") {
        (FailureKind::Unreachable, "Host unreachable".to_string())
    } else if lower.contains("timeout") || lower.contains("timed out") {
        (FailureKind::Timeout, "Ping timeout".to_string())
    } else if lower.contains("unknown host") || lower.contains("cannot resolve") {
        (FailureKind::Dns, "DNS resolution failed".to_string())
    } else {
        (FailureKind::Unknown, "Ping failed".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_icmp_ping_localhost() {
        let result = icmp_ping("127.0.0.1", Duration::from_secs(2)).await;
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
}
