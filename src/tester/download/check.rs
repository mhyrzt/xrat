use reqwest::Proxy;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::model::Node;
use crate::tester::FailureKind;
use crate::xray::{XrayProcess, generate_probe_config};

use super::errors::{classify_request_error, classify_xray_error};

#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub success: bool,
    pub mbps: Option<f64>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

pub async fn download_speed_check(
    node: &Node,
    test_url: &str,
    xray_binary_path: &Path,
    xray_startup_timeout: Duration,
    request_timeout: Duration,
) -> DownloadResult {
    let local_port = match find_available_port().await {
        Ok(port) => port,
        Err(error) => {
            return DownloadResult {
                success: false,
                mbps: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to find available port: {error}")),
            };
        }
    };

    let config = match generate_probe_config(node, local_port) {
        Ok(config) => config,
        Err(error) => {
            return DownloadResult {
                success: false,
                mbps: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to generate config: {error}")),
            };
        }
    };

    let process =
        match XrayProcess::spawn_with_binary(xray_binary_path, &config, xray_startup_timeout).await
        {
            Ok(process) => process,
            Err(error) => {
                let (kind, reason) = classify_xray_error(&error);
                return DownloadResult {
                    success: false,
                    mbps: None,
                    failure_kind: Some(kind),
                    failure_reason: Some(reason),
                };
            }
        };

    let result = make_proxied_download(local_port, test_url, request_timeout).await;
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

async fn make_proxied_download(
    proxy_port: u16,
    test_url: &str,
    timeout_duration: Duration,
) -> DownloadResult {
    let proxy_url = format!("socks5://127.0.0.1:{proxy_port}");
    let proxy = match Proxy::all(&proxy_url) {
        Ok(proxy) => proxy,
        Err(error) => {
            return DownloadResult {
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
            return DownloadResult {
                success: false,
                mbps: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to create HTTP client: {error}")),
            };
        }
    };

    let response = match client.get(test_url).send().await {
        Ok(response) => response,
        Err(error) => {
            let (kind, reason) = classify_request_error(&error);
            return DownloadResult {
                success: false,
                mbps: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            };
        }
    };

    if !response.status().is_success() {
        return DownloadResult {
            success: false,
            mbps: None,
            failure_kind: Some(FailureKind::Proxy),
            failure_reason: Some(format!("HTTP status: {}", response.status())),
        };
    }

    let start = Instant::now();
    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => {
            let (kind, reason) = classify_request_error(&error);
            return DownloadResult {
                success: false,
                mbps: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            };
        }
    };

    if bytes.is_empty() {
        return DownloadResult {
            success: false,
            mbps: None,
            failure_kind: Some(FailureKind::Proxy),
            failure_reason: Some("Download response was empty".to_string()),
        };
    }

    DownloadResult {
        success: true,
        mbps: Some(calculate_mbps(bytes.len() as u64, start.elapsed())),
        failure_kind: None,
        failure_reason: None,
    }
}

pub(super) fn calculate_mbps(byte_count: u64, elapsed: Duration) -> f64 {
    let seconds = elapsed.as_secs_f64().max(f64::EPSILON);
    (byte_count as f64 * 8.0) / seconds / 1_000_000.0
}
