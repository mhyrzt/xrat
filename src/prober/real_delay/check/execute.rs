use std::path::Path;
use std::time::Duration;

use super::super::errors::classify_xray_error;
use super::model::RealDelayResult;
use super::port::find_available_port;
use super::request::make_proxied_request;
use crate::model::Node;
use crate::prober::FailureKind;
use crate::prober::real_delay::AcceptedHttpStatuses;
use crate::xray::{XrayGenOptions, XrayProcess, generate_probe_config_with_options};

#[allow(clippy::too_many_arguments)]
pub async fn real_delay_check(
    node: &Node,
    test_url: &str,
    xray_binary_path: &Path,
    xray_startup_timeout: Duration,
    request_timeout: Duration,
    gen_options: &XrayGenOptions,
    accepted_statuses: &AcceptedHttpStatuses,
    follow_redirects: bool,
) -> RealDelayResult {
    let local_port = match find_available_port().await {
        Ok(port) => port,
        Err(error) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                ttfb_ms: None,
                http_status: None,
                dial_endpoint_ip: None,
                failure_kind: Some(FailureKind::Process),
                failure_reason: Some(format!("Failed to find available port: {error}")),
            };
        }
    };

    let config = match generate_probe_config_with_options(node, local_port, gen_options) {
        Ok(config) => config,
        Err(error) => {
            return RealDelayResult {
                success: false,
                latency_ms: None,
                ttfb_ms: None,
                http_status: None,
                dial_endpoint_ip: None,
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
                return RealDelayResult {
                    success: false,
                    latency_ms: None,
                    ttfb_ms: None,
                    http_status: None,
                    dial_endpoint_ip: None,
                    failure_kind: Some(kind),
                    failure_reason: Some(reason),
                };
            }
        };

    let result = make_proxied_request(
        local_port,
        test_url,
        request_timeout,
        accepted_statuses,
        follow_redirects,
    )
    .await;
    let _ = process.kill();
    result
}
