use reqwest::Proxy;
use std::time::{Duration, Instant};

use crate::prober::FailureKind;

use super::result::DownloadResult;
use crate::prober::download::errors::classify_request_error;

pub(super) async fn find_available_port() -> Result<u16, std::io::Error> {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    drop(listener);
    Ok(port)
}

pub(super) async fn make_proxied_download(
    proxy_port: u16,
    test_url: &str,
    timeout_duration: Duration,
) -> DownloadResult {
    let proxy_url = format!("socks5://127.0.0.1:{proxy_port}");
    let proxy = match Proxy::all(&proxy_url) {
        Ok(proxy) => proxy,
        Err(error) => {
            return DownloadResult::process_failure(format!(
                "Failed to create proxy config: {error}"
            ));
        }
    };

    let client = match reqwest::Client::builder()
        .proxy(proxy)
        .timeout(timeout_duration)
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            return DownloadResult::process_failure(format!(
                "Failed to create HTTP client: {error}"
            ));
        }
    };

    let response = match client.get(test_url).send().await {
        Ok(response) => response,
        Err(error) => {
            let (kind, reason) = classify_request_error(&error);
            return DownloadResult::failure(kind, reason);
        }
    };

    if !response.status().is_success() {
        return DownloadResult::failure(
            FailureKind::Proxy,
            format!("HTTP status: {}", response.status()),
        );
    }

    let start = Instant::now();
    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => {
            let (kind, reason) = classify_request_error(&error);
            return DownloadResult::failure(kind, reason);
        }
    };

    if bytes.is_empty() {
        return DownloadResult::failure(
            FailureKind::Proxy,
            "Download response was empty".to_string(),
        );
    }

    DownloadResult::success(bytes.len() as u64, start.elapsed())
}
