use reqwest::Proxy;
use std::path::Path;
use std::time::{Duration, Instant};

use super::errors::{classify_request_error, classify_xray_error};
use crate::model::Node;
use crate::tester::FailureKind;
use crate::xray::{XrayProcess, generate_probe_config};

#[derive(Debug, Clone)]
pub struct RealDelayResult {
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub ttfb_ms: Option<u32>,
    pub http_status: Option<u16>,
    pub endpoint_ip: Option<String>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

pub async fn real_delay_check(
    node: &Node,
    test_url: &str,
    xray_binary_path: &Path,
    xray_startup_timeout: Duration,
    request_timeout: Duration,
) -> RealDelayResult {
    let local_port = match find_available_port().await {
        Ok(port) => port,
        Err(e) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                ttfb_ms: None,
                http_status: None,
                endpoint_ip: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to find available port: {}", e)),
            };
        }
    };

    let config = match generate_probe_config(node, local_port) {
        Ok(cfg) => cfg,
        Err(e) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                ttfb_ms: None,
                http_status: None,
                endpoint_ip: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to generate config: {}", e)),
            };
        }
    };

    let process =
        match XrayProcess::spawn_with_binary(xray_binary_path, &config, xray_startup_timeout).await
        {
            Ok(proc) => proc,
            Err(e) => {
                let (kind, reason) = classify_xray_error(&e);
                return RealDelayResult {
                    success: false,
                    latency_ms: None,
                    ttfb_ms: None,
                    http_status: None,
                    endpoint_ip: None,
                    failure_kind: Some(kind),
                    failure_reason: Some(reason),
                };
            }
        };

    let result = make_proxied_request(local_port, test_url, request_timeout).await;
    let _ = process.kill();
    result
}

pub(super) async fn find_available_port() -> Result<u16, std::io::Error> {
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
                ttfb_ms: None,
                http_status: None,
                endpoint_ip: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to create HTTP client: {}", e)),
            };
        }
    };

    let start = Instant::now();
    match client.get(test_url).send().await {
        Ok(response) => {
            let ttfb_ms = start.elapsed().as_millis() as u32;
            let status = response.status().as_u16();
            let endpoint_ip = response.remote_addr().map(|addr| addr.ip().to_string());
            if response.status().is_success() || response.status().as_u16() == 204 {
                RealDelayResult {
                    success: true,
                    latency_ms: Some(ttfb_ms),
                    ttfb_ms: Some(ttfb_ms),
                    http_status: Some(status),
                    endpoint_ip,
                    failure_kind: None,
                    failure_reason: None,
                }
            } else {
                RealDelayResult {
                    success: false,
                    latency_ms: None,
                    ttfb_ms: Some(ttfb_ms),
                    http_status: Some(status),
                    endpoint_ip,
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
                ttfb_ms: None,
                http_status: None,
                endpoint_ip: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            }
        }
    }
}
