use std::path::Path;
use std::time::Duration;

use crate::model::Node;
use crate::xray::{XrayGenOptions, generate_probe_config_with_options};

use super::FailureKind;

mod classify;
mod request;

pub use classify::{classify_request_error, classify_xray_error};
use request::{find_available_port, make_proxied_upload};

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub success: bool,
    pub mbps: Option<f64>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn upload_speed_check(
    node: &Node,
    test_url: &str,
    xray_binary_path: &Path,
    xray_startup_timeout: Duration,
    request_timeout: Duration,
    payload_bytes: usize,
    gen_options: &XrayGenOptions,
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

    let config = match generate_probe_config_with_options(node, local_port, gen_options) {
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

    let process = match crate::xray::XrayProcess::spawn_with_binary(
        xray_binary_path,
        &config,
        xray_startup_timeout,
    )
    .await
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
