use reqwest::Proxy;
use std::time::{Duration, Instant};

use super::super::errors::classify_request_error;
use super::model::RealDelayResult;
use crate::prober::FailureKind;

pub(crate) async fn make_proxied_request(
    proxy_port: u16,
    test_url: &str,
    timeout_duration: Duration,
) -> RealDelayResult {
    let proxy_url = format!("socks5://127.0.0.1:{proxy_port}");
    let proxy = match Proxy::all(&proxy_url) {
        Ok(proxy) => proxy,
        Err(error) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                ttfb_ms: None,
                http_status: None,
                endpoint_ip: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to create proxy config: {error}")),
            };
        }
    };

    let client = match reqwest::Client::builder()
        .proxy(proxy)
        .timeout(timeout_duration)
        .build()
    {
        Ok(c) => c,
        Err(error) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                ttfb_ms: None,
                http_status: None,
                endpoint_ip: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to create HTTP client: {error}")),
            };
        }
    };

    let start = Instant::now();
    match client.get(test_url).send().await {
        Ok(response) => {
            let ttfb_ms = start.elapsed().as_millis() as u32;
            let status = response.status().as_u16();
            let endpoint_ip = response.remote_addr().map(|addr| addr.ip().to_string());
            if response.status().is_success() || status == 204 {
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
        Err(error) => {
            let (kind, reason) = classify_request_error(&error);
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
