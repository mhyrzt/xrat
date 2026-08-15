use reqwest::redirect::Policy;
use reqwest::{Client, Proxy};
use std::time::{Duration, Instant};

use super::super::errors::classify_request_error;
use super::model::RealDelayResult;
use crate::prober::FailureKind;
use crate::prober::real_delay::AcceptedHttpStatuses;

pub(crate) const MAX_REDIRECTS: usize = 10;

pub(crate) fn redirect_policy(follow_redirects: bool) -> Policy {
    if follow_redirects {
        Policy::limited(MAX_REDIRECTS)
    } else {
        Policy::none()
    }
}

pub(crate) async fn make_proxied_request(
    proxy_port: u16,
    test_url: &str,
    timeout_duration: Duration,
    accepted_statuses: &AcceptedHttpStatuses,
    follow_redirects: bool,
) -> RealDelayResult {
    make_proxied_request_via(
        &format!("socks5h://127.0.0.1:{proxy_port}"),
        test_url,
        timeout_duration,
        accepted_statuses,
        follow_redirects,
    )
    .await
}

pub(crate) async fn make_proxied_request_via(
    proxy_url: &str,
    test_url: &str,
    timeout_duration: Duration,
    accepted_statuses: &AcceptedHttpStatuses,
    follow_redirects: bool,
) -> RealDelayResult {
    let proxy = match Proxy::all(proxy_url) {
        Ok(proxy) => proxy,
        Err(error) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                ttfb_ms: None,
                http_status: None,
                dial_endpoint_ip: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to create proxy config: {error}")),
            };
        }
    };

    let client = match reqwest::Client::builder()
        .proxy(proxy)
        .timeout(timeout_duration)
        .redirect(redirect_policy(follow_redirects))
        .build()
    {
        Ok(c) => c,
        Err(error) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                ttfb_ms: None,
                http_status: None,
                dial_endpoint_ip: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to create HTTP client: {error}")),
            };
        }
    };

    make_request(&client, test_url, accepted_statuses).await
}

pub(crate) async fn make_request(
    client: &Client,
    test_url: &str,
    accepted_statuses: &AcceptedHttpStatuses,
) -> RealDelayResult {
    let start = Instant::now();
    match client.get(test_url).send().await {
        Ok(response) => {
            let ttfb_ms = start.elapsed().as_millis() as u32;
            let status = response.status().as_u16();
            let dial_endpoint_ip = response.remote_addr().map(|addr| addr.ip().to_string());
            if accepted_statuses.matches(status) {
                RealDelayResult {
                    success: true,
                    latency_ms: Some(ttfb_ms),
                    ttfb_ms: Some(ttfb_ms),
                    http_status: Some(status),
                    dial_endpoint_ip,
                    failure_kind: None,
                    failure_reason: None,
                }
            } else {
                RealDelayResult {
                    success: false,
                    latency_ms: None,
                    ttfb_ms: Some(ttfb_ms),
                    http_status: Some(status),
                    dial_endpoint_ip,
                    failure_kind: Some(FailureKind::Proxy),
                    failure_reason: Some(format!(
                        "HTTP status {status} from {test_url} is not accepted; expected {}",
                        accepted_statuses.description()
                    )),
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
                dial_endpoint_ip: None,
                failure_kind: Some(kind),
                failure_reason: Some(reason),
            }
        }
    }
}
