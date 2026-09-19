use std::path::Path;
use std::time::Duration;

use super::model::RealDelayResult;
use super::port::find_available_port;
use super::request::make_proxied_request;
use crate::model::Node;
use crate::prober::FailureKind;
use crate::prober::probe::{ProbeEngineKind, ProbeProcess};
use crate::prober::real_delay::AcceptedHttpStatuses;
use crate::xray::XrayGenOptions;

#[allow(clippy::too_many_arguments)]
pub async fn real_delay_check(
    node: &Node,
    test_url: &str,
    engine: ProbeEngineKind,
    binary_path: &Path,
    startup_timeout: Duration,
    request_timeout: Duration,
    gen_options: &XrayGenOptions,
    accepted_statuses: &AcceptedHttpStatuses,
    follow_redirects: bool,
) -> RealDelayResult {
    let local_port = match find_available_port().await {
        Ok(port) => port,
        Err(error) => {
            return RealDelayResult::failure(
                FailureKind::Process,
                format!("Failed to find available port: {error}"),
            );
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
        Err((kind, reason)) => return RealDelayResult::failure(kind, reason),
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
