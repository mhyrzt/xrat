use super::*;

mod resolve;
mod rows;
mod validation;

pub(crate) use resolve::resolve_test_settings;
pub(crate) use rows::TestOutputRow;
pub(crate) use validation::{resolve_concurrency, validate_test_stage_order};

impl TestFailurePolicy {
    pub(crate) fn halts_after_failure(self) -> bool {
        matches!(self, Self::SkipRemaining | Self::MarkFailed)
    }
}

pub(crate) fn resolve_engine_binary_path(
    app_config: &AppConfig,
    runtime_paths: &RuntimePaths,
) -> PathBuf {
    match app_config.runtime.engine.as_str() {
        "sing-box" => runtime_paths.sing_box_path.clone(),
        "v2ray" => runtime_paths.v2ray_path.clone(),
        "xray" => runtime_paths.xray_path.clone(),
        other => PathBuf::from(other),
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ResolvedTestSettings {
    pub(crate) stage_order: Vec<ConnectionTestStage>,
    pub(crate) failure_policy: TestFailurePolicy,
    pub(crate) real_delay_url: String,
    pub(crate) download_url: String,
    pub(crate) upload_url: Option<String>,
    pub(crate) xray_binary_path: PathBuf,
    pub(crate) icmp_timeout: Duration,
    pub(crate) tcp_timeout: Duration,
    pub(crate) xray_startup_timeout: Duration,
    pub(crate) real_delay_timeout: Duration,
    pub(crate) download_timeout: Duration,
    pub(crate) upload_timeout: Duration,
    pub(crate) upload_payload_bytes: usize,
    pub(crate) run_icmp: bool,
    pub(crate) run_tcp: bool,
    pub(crate) run_real_delay: bool,
    pub(crate) run_download: bool,
    pub(crate) run_upload: bool,
    pub(crate) concurrency: i32,
    pub(crate) geoip_enabled: bool,
    pub(crate) geoip_country_path: PathBuf,
    pub(crate) geoip_city_path: PathBuf,
    pub(crate) geoip_asn_path: PathBuf,
    pub(crate) geoip_lookup: Arc<dyn GeoIpLookup>,
    pub(crate) gen_options: XrayGenOptions,
}
use std::sync::Arc;

use crate::support::geoip::GeoIpLookup;
use crate::xray::XrayGenOptions;
