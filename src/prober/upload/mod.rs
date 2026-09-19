use std::path::Path;
use std::time::Duration;

use crate::model::Node;
use crate::prober::probe::{ProbeEngineKind, ProbeProcess};
use crate::xray::XrayGenOptions;

use super::FailureKind;

mod classify;
mod request;

pub use classify::classify_request_error;
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
    engine: ProbeEngineKind,
    binary_path: &Path,
    startup_timeout: Duration,
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

    let process = match ProbeProcess::spawn(
        node,
        local_port,
        engine,
        binary_path,
        gen_options,
        startup_timeout,
    )
    .await
    {
        Ok(process) => process,
        Err((kind, reason)) => {
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
