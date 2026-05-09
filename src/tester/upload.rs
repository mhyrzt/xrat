use reqwest::Proxy;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::model::Node;
use crate::xray::{XrayProcess, XrayProcessError, generate_probe_config};

use super::FailureKind;

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub success: bool,
    pub mbps: Option<f64>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

pub async fn upload_speed_check(
    node: &Node,
    test_url: &str,
    xray_binary_path: &Path,
    xray_startup_timeout: Duration,
    request_timeout: Duration,
    payload_bytes: usize,
) -> UploadResult {
    let local_port = match find_available_port().await {
        Ok(port) => port,
        Err(error) => {
            return UploadResult {
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
            return UploadResult {
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
                return UploadResult {
                    success: false,
                    mbps: None,
                    failure_kind: Some(kind),
                    failure_reason: Some(reason),
                };
            }
        };

    let result = make_proxied_upload(local_port, test_url, request_timeout, payload_bytes).await;
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

async fn make_proxied_upload(
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

fn classify_xray_error(error: &XrayProcessError) -> (FailureKind, String) {
    match error {
        XrayProcessError::SpawnError(_) => (
            FailureKind::Process,
            format!("Failed to spawn xray: {error}"),
        ),
        XrayProcessError::StartupTimeout => {
            (FailureKind::Timeout, "Xray startup timeout".to_string())
        }
        XrayProcessError::ProcessExited(_) => (
            FailureKind::Process,
            "Xray process exited unexpectedly".to_string(),
        ),
        XrayProcessError::PortNotReady(_) => (
            FailureKind::Process,
            format!("Xray port not ready: {error}"),
        ),
        _ => (FailureKind::Process, format!("Xray error: {error}")),
    }
}

fn classify_request_error(error: &reqwest::Error) -> (FailureKind, String) {
    if error.is_timeout() {
        (FailureKind::Timeout, "Upload request timeout".to_string())
    } else if error.to_string().to_lowercase().contains("tls") {
        (FailureKind::Tls, format!("TLS handshake failed: {error}"))
    } else if error.to_string().to_lowercase().contains("407") {
        (
            FailureKind::Auth,
            format!("Proxy authentication failed: {error}"),
        )
    } else if error.is_connect() {
        (
            FailureKind::Proxy,
            format!("Proxy connection failed: {error}"),
        )
    } else if error.is_request() {
        (
            FailureKind::Proxy,
            format!("Upload request failed: {error}"),
        )
    } else {
        (FailureKind::Unknown, format!("Upload HTTP error: {error}"))
    }
}
