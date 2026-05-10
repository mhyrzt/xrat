use reqwest::Proxy;
use std::time::{Duration, Instant};

use super::{FailureKind, UploadResult, classify_request_error};

pub async fn find_available_port() -> Result<u16, std::io::Error> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

pub async fn make_proxied_upload(
    proxy_port: u16,
    test_url: &str,
    timeout_duration: Duration,
    payload_bytes: usize,
) -> UploadResult {
    let proxy_url = format!("socks5://127.0.0.1:{proxy_port}");
    let proxy = match Proxy::all(&proxy_url) {
        Ok(proxy) => proxy,
        Err(error) => {
            return UploadResult {
                success: false,
                mbps: None,
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
        Ok(client) => client,
        Err(error) => {
            return UploadResult {
                success: false,
                mbps: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to create HTTP client: {error}")),
            };
        }
    };

    let payload = vec![0_u8; payload_bytes.max(1024)];
    let start = Instant::now();
    let response = match client.post(test_url).body(payload.clone()).send().await {
        Ok(response) => response,
        Err(error) => {
            let (kind, reason) = classify_request_error(&error);
            return UploadResult {
                success: false,
                mbps: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            };
        }
    };

    if !response.status().is_success() {
        return UploadResult {
            success: false,
            mbps: None,
            failure_kind: Some(FailureKind::Proxy),
            failure_reason: Some(format!("HTTP status: {}", response.status())),
        };
    }

    UploadResult {
        success: true,
        mbps: Some(calculate_mbps(payload.len() as u64, start.elapsed())),
        failure_kind: None,
        failure_reason: None,
    }
}

fn calculate_mbps(byte_count: u64, elapsed: Duration) -> f64 {
    let seconds = elapsed.as_secs_f64().max(f64::EPSILON);
    (byte_count as f64 * 8.0) / seconds / 1_000_000.0
}
