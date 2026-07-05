use std::path::Path;
use std::time::Duration;

use crate::model::Node;
use crate::xray::{XrayGenOptions, XrayProcess, generate_probe_config_with_options};

use super::errors::classify_xray_error;

mod proxied;
mod result;

pub use result::DownloadResult;
#[cfg(test)]
pub(crate) use result::calculate_mbps;

#[allow(clippy::too_many_arguments)]
pub async fn download_speed_check(
    node: &Node,
    test_url: &str,
    xray_binary_path: &Path,
    xray_startup_timeout: Duration,
    request_timeout: Duration,
    gen_options: &XrayGenOptions,
) -> DownloadResult {
    let local_port = match proxied::find_available_port().await {
        Ok(port) => port,
        Err(error) => {
            return DownloadResult::process_failure(format!(
                "Failed to find available port: {error}"
            ));
        }
    };

    let config = match generate_probe_config_with_options(node, local_port, gen_options) {
        Ok(config) => config,
        Err(error) => {
            return DownloadResult::process_failure(format!("Failed to generate config: {error}"));
        }
    };

    let process =
        match XrayProcess::spawn_with_binary(xray_binary_path, &config, xray_startup_timeout).await
        {
            Ok(process) => process,
            Err(error) => {
                let (kind, reason) = classify_xray_error(&error);
                return DownloadResult::failure(kind, reason);
            }
        };

    let result = proxied::make_proxied_download(local_port, test_url, request_timeout).await;
    let _ = process.kill();

    result
}
