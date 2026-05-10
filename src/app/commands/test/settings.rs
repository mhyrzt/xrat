use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResolvedTestSettings {
    pub(super) stage_order: Vec<ConnectionTestStage>,
    pub(super) failure_policy: TestFailurePolicy,
    pub(super) real_delay_url: String,
    pub(super) download_url: String,
    pub(super) upload_url: Option<String>,
    pub(super) xray_binary_path: PathBuf,
    pub(super) icmp_timeout: Duration,
    pub(super) tcp_timeout: Duration,
    pub(super) xray_startup_timeout: Duration,
    pub(super) real_delay_timeout: Duration,
    pub(super) download_timeout: Duration,
    pub(super) upload_timeout: Duration,
    pub(super) upload_payload_bytes: usize,
    pub(super) run_icmp: bool,
    pub(super) run_tcp: bool,
    pub(super) run_real_delay: bool,
    pub(super) run_download: bool,
    pub(super) run_upload: bool,
    pub(super) concurrency: i32,
    pub(super) geoip_enabled: bool,
    pub(super) geoip_country_path: PathBuf,
    pub(super) geoip_city_path: PathBuf,
    pub(super) geoip_asn_path: PathBuf,
}

pub(super) fn resolve_test_settings(
    args: &TestArgs,
    app_config: &AppConfig,
    runtime_paths: &RuntimePaths,
) -> crate::app::Result<ResolvedTestSettings> {
    let concurrency = args.concurrency.unwrap_or(app_config.testing.concurrency);
    if concurrency < 0 {
        return Err(AppError::InvalidArgument(
            "test concurrency must be 0 or greater".to_string(),
        ));
    }
    validate_test_stage_order(&app_config.testing.order)?;

    Ok(ResolvedTestSettings {
        stage_order: app_config.testing.order.clone(),
        failure_policy: app_config.testing.failure_policy,
        real_delay_url: args
            .test_url
            .clone()
            .unwrap_or_else(|| app_config.testing.real_delay.url.clone()),
        download_url: args
            .download_url
            .clone()
            .unwrap_or_else(|| app_config.testing.download.url.clone()),
        upload_url: args.upload_url.clone(),
        xray_binary_path: resolve_engine_binary_path(app_config, runtime_paths),
        icmp_timeout: Duration::from_millis(
            args.icmp_timeout_ms
                .unwrap_or(app_config.testing.icmp.timeout),
        ),
        tcp_timeout: Duration::from_millis(
            args.tcp_timeout_ms
                .unwrap_or(app_config.testing.tcp.timeout),
        ),
        xray_startup_timeout: Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
        real_delay_timeout: Duration::from_millis(
            args.real_delay_timeout_ms
                .unwrap_or(app_config.testing.real_delay.timeout),
        ),
        download_timeout: Duration::from_millis(
            args.download_timeout_ms
                .unwrap_or(app_config.testing.download.timeout),
        ),
        upload_timeout: Duration::from_millis(
            args.upload_timeout_ms
                .unwrap_or(defaults::DEFAULT_UPLOAD_TIMEOUT_MS),
        ),
        upload_payload_bytes: defaults::DEFAULT_UPLOAD_PAYLOAD_BYTES,
        run_icmp: app_config.testing.icmp.enabled && !args.skip_icmp,
        run_tcp: app_config.testing.tcp.enabled && !args.skip_tcp,
        run_real_delay: app_config.testing.real_delay.enabled && !args.skip_real_delay,
        run_download: app_config.testing.download.enabled && !args.skip_download,
        run_upload: args.upload_url.is_some() && !args.skip_upload,
        concurrency,
        geoip_enabled: app_config.testing.geoip.enabled,
        geoip_country_path: config::resolve_config_path(
            &runtime_paths.config_path,
            &app_config.testing.geoip.country_path,
        ),
        geoip_city_path: config::resolve_config_path(
            &runtime_paths.config_path,
            &app_config.testing.geoip.city_path,
        ),
        geoip_asn_path: config::resolve_config_path(
            &runtime_paths.config_path,
            &app_config.testing.geoip.asn_path,
        ),
    })
}

pub(super) fn validate_test_stage_order(order: &[ConnectionTestStage]) -> crate::app::Result<()> {
    let mut seen = Vec::with_capacity(order.len());
    for stage in order {
        if seen.contains(stage) {
            return Err(AppError::InvalidArgument(format!(
                "duplicate test stage in [testing].order: {}",
                test_stage_name(*stage)
            )));
        }
        seen.push(*stage);
    }

    Ok(())
}

pub(super) fn test_stage_name(stage: ConnectionTestStage) -> &'static str {
    match stage {
        ConnectionTestStage::Icmp => "icmp",
        ConnectionTestStage::RealDelay => "real_delay",
        ConnectionTestStage::Download => "download",
    }
}

impl TestFailurePolicy {
    pub(super) fn halts_after_failure(self) -> bool {
        matches!(self, Self::SkipRemaining | Self::MarkFailed)
    }
}

pub(super) fn resolve_engine_binary_path(
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

#[derive(Clone, Debug, Serialize)]
pub(super) struct TestOutputRow {
    pub(super) id: i64,
    pub(super) name: Option<String>,
    pub(super) protocol: String,
    pub(super) address: String,
    pub(super) port: i64,
    pub(super) icmp_ms: Option<u32>,
    pub(super) real_delay_ms: Option<u32>,
    pub(super) download_mbps: Option<f64>,
    pub(super) upload_mbps: Option<f64>,
    pub(super) status: TestStatus,
    pub(super) error: Option<String>,
    #[serde(skip_serializing)]
    pub(super) tcp_ms: Option<u32>,
    #[serde(skip_serializing)]
    pub(super) ttfb_ms: Option<u32>,
    #[serde(skip_serializing)]
    pub(super) http_status: Option<u16>,
    #[serde(skip_serializing)]
    pub(super) endpoint_ip: Option<String>,
    #[serde(skip_serializing)]
    pub(super) endpoint_location: Option<String>,
    #[serde(skip_serializing)]
    pub(super) endpoint_country: Option<String>,
    #[serde(skip_serializing)]
    pub(super) endpoint_asn: Option<String>,
    #[serde(skip_serializing)]
    pub(super) ran_icmp: bool,
    #[serde(skip_serializing)]
    pub(super) ran_tcp: bool,
    #[serde(skip_serializing)]
    pub(super) ran_real_delay: bool,
    #[serde(skip_serializing)]
    pub(super) icmp_ok: bool,
    #[serde(skip_serializing)]
    pub(super) tcp_ok: bool,
    #[serde(skip_serializing)]
    pub(super) real_delay_ok: bool,
    #[serde(skip_serializing)]
    pub(super) failure_kind: Option<String>,
    #[serde(skip_serializing)]
    pub(super) elapsed_secs: f64,
}
