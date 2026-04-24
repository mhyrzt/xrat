use reqwest::Proxy;
use std::time::{Duration, Instant};

use crate::model::Node;
use crate::xray::{XrayProcess, XrayProcessError, generate_probe_config};

use super::FailureKind;

#[derive(Debug, Clone)]
pub struct RealDelayResult {
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

/// Default test target URL
pub const DEFAULT_TEST_URL: &str = "https://www.gstatic.com/generate_204";

/// Perform real-delay check through actual proxy traffic
pub async fn real_delay_check(
    node: &Node,
    test_url: &str,
    xray_startup_timeout: Duration,
    request_timeout: Duration,
) -> RealDelayResult {
    // Find an available ephemeral port
    let local_port = match find_available_port().await {
        Ok(port) => port,
        Err(e) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to find available port: {}", e)),
            };
        }
    };

    // Generate probe config
    let config = match generate_probe_config(node, local_port) {
        Ok(cfg) => cfg,
        Err(e) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to generate config: {}", e)),
            };
        }
    };

    // Spawn Xray process
    let process = match XrayProcess::spawn(&config, xray_startup_timeout).await {
        Ok(proc) => proc,
        Err(e) => {
            let (kind, reason) = classify_xray_error(&e);
            return RealDelayResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            };
        }
    };

    // Make HTTP request through the proxy
    let result = make_proxied_request(local_port, test_url, request_timeout).await;

    // Clean up process
    let _ = process.kill();

    result
}

async fn find_available_port() -> Result<u16, std::io::Error> {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

async fn make_proxied_request(
    proxy_port: u16,
    test_url: &str,
    timeout_duration: Duration,
) -> RealDelayResult {
    let proxy_url = format!("socks5://127.0.0.1:{}", proxy_port);

    let client = match reqwest::Client::builder()
        .proxy(Proxy::all(&proxy_url).unwrap())
        .timeout(timeout_duration)
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to create HTTP client: {}", e)),
            };
        }
    };

    let start = Instant::now();
    match client.get(test_url).send().await {
        Ok(response) => {
            let elapsed = start.elapsed();
            if response.status().is_success() || response.status().as_u16() == 204 {
                RealDelayResult {
                    success: true,
                    latency_ms: Some(elapsed.as_millis() as u32),
                    failure_kind: None,
                    failure_reason: None,
                }
            } else {
                RealDelayResult {
                    success: false,
                    latency_ms: None,
                    failure_kind: Some(FailureKind::Proxy),
                    failure_reason: Some(format!("HTTP status: {}", response.status())),
                }
            }
        }
        Err(e) => {
            let (kind, reason) = classify_request_error(&e);
            RealDelayResult {
                success: false,
                latency_ms: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            }
        }
    }
}

fn classify_xray_error(error: &XrayProcessError) -> (FailureKind, String) {
    match error {
        XrayProcessError::SpawnError(_) => (
            FailureKind::Process,
            format!("Failed to spawn xray: {}", error),
        ),
        XrayProcessError::StartupTimeout => {
            (FailureKind::Timeout, "Xray startup timeout".to_string())
        }
        XrayProcessError::ProcessExited => (
            FailureKind::Process,
            "Xray process exited unexpectedly".to_string(),
        ),
        XrayProcessError::PortNotReady(_) => (
            FailureKind::Process,
            format!("Xray port not ready: {}", error),
        ),
        _ => (FailureKind::Process, format!("Xray error: {}", error)),
    }
}

fn classify_request_error(error: &reqwest::Error) -> (FailureKind, String) {
    if error.is_timeout() {
        (FailureKind::Timeout, "Request timeout".to_string())
    } else if error.is_connect() {
        (
            FailureKind::Proxy,
            format!("Proxy connection failed: {}", error),
        )
    } else if error.is_request() {
        (FailureKind::Proxy, format!("Request failed: {}", error))
    } else {
        (FailureKind::Unknown, format!("HTTP error: {}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Protocol;

    #[tokio::test]
    async fn test_find_available_port() {
        let port = find_available_port().await.unwrap();
        assert!(port > 0);
    }

    #[tokio::test]
    async fn test_real_delay_check_invalid_config() {
        // Test with an invalid node (missing required fields)
        let node = Node {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: None, // Missing required UUID
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: None,
            sni: None,
            host: None,
            path: None,
            name: Some("test".to_string()),
            raw_config: "".to_string(),
        };

        let result = real_delay_check(
            &node,
            DEFAULT_TEST_URL,
            Duration::from_secs(5),
            Duration::from_secs(10),
        )
        .await;

        assert!(!result.success);
        assert!(matches!(result.failure_kind, Some(FailureKind::Process)));
    }
}
