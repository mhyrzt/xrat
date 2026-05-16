use std::net::IpAddr;
use std::time::{Duration, Instant};
use tokio::process::Command;

use super::FailureKind;

mod parsing;

#[cfg(test)]
mod tests;

pub use parsing::{classify_ping_failure, parse_ping_latency};

#[derive(Debug, Clone)]
pub struct IcmpResult {
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

pub async fn icmp_ping(address: &str, timeout: Duration) -> IcmpResult {
    let ip = match resolve_address(address).await {
        Ok(ip) => ip,
        Err(error) => {
            return IcmpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(FailureKind::Dns),
                failure_reason: Some(error),
            };
        }
    };

    ping_with_system_command(&ip.to_string(), timeout).await
}

async fn resolve_address(address: &str) -> Result<IpAddr, String> {
    if let Ok(ip) = address.parse::<IpAddr>() {
        return Ok(ip);
    }

    tokio::net::lookup_host(format!("{address}:0"))
        .await
        .map_err(|error| format!("DNS resolution failed: {error}"))?
        .next()
        .map(|addr| addr.ip())
        .ok_or_else(|| "No IP address found".to_string())
}

async fn ping_with_system_command(ip: &str, timeout: Duration) -> IcmpResult {
    let timeout_secs = timeout.as_secs().max(1).to_string();
    let (count_flag, timeout_flag) = ping_flags();

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
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let latency = parse_ping_latency(&stdout).unwrap_or(elapsed.as_millis() as u32);
            IcmpResult {
                success: true,
                latency_ms: Some(latency),
                failure_kind: None,
                failure_reason: None,
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let (kind, reason) = classify_ping_failure(&format!("{stdout}{stderr}"));
            IcmpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            }
        }
        Err(error) => {
            let kind = if error.kind() == std::io::ErrorKind::PermissionDenied {
                FailureKind::PermissionDenied
            } else {
                FailureKind::Unknown
            };
            IcmpResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(kind),
                failure_reason: Some(format!("Failed to execute ping: {error}")),
            }
        }
    }
}

fn ping_flags() -> (&'static str, &'static str) {
    if cfg!(target_os = "macos") {
        ("-c", "-t")
    } else if cfg!(target_os = "linux") {
        ("-c", "-W")
    } else if cfg!(target_os = "windows") {
        ("-n", "-w")
    } else {
        ("-c", "-W")
    }
}
