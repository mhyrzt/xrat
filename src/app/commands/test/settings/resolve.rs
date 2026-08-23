use super::*;
use crate::app::paths::mmdb;
use crate::support::geoip;

pub(crate) fn resolve_test_settings(
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
    let real_delay = &app_config.testing.real_delay;
    let accepted_http_statuses = if real_delay.accepted_status_codes.is_none()
        && real_delay.accepted_status_ranges.is_none()
    {
        AcceptedHttpStatuses::default()
    } else {
        AcceptedHttpStatuses::new(
            real_delay.accepted_status_codes.clone().unwrap_or_default(),
            real_delay
                .accepted_status_ranges
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|range| range.bounds())
                .collect(),
        )
        .map_err(|error| {
            AppError::InvalidArgument(format!(
                "invalid [testing.real_delay] status acceptance: {error}"
            ))
        })?
    };
    let geoip_country_path = mmdb::mmdb_path_for(
        runtime_paths,
        app_config,
        &app_config.testing.geoip.country_path,
        "GeoLite2-Country.mmdb",
    );
    let geoip_city_path = mmdb::mmdb_path_for(
        runtime_paths,
        app_config,
        &app_config.testing.geoip.city_path,
        "GeoLite2-City.mmdb",
    );
    let geoip_asn_path = mmdb::mmdb_path_for(
        runtime_paths,
        app_config,
        &app_config.testing.geoip.asn_path,
        "GeoLite2-ASN.mmdb",
    );
    let geoip_lookup = geoip::build_lookup_chain(app_config, runtime_paths)?;
    let mut gen_options = crate::app::runtime_tuning::build_xray_gen_options(&app_config.runtime);
    crate::app::runtime_tuning::apply_xray_dns_options(&mut gen_options, &app_config.dns)?;

    Ok(ResolvedTestSettings {
        stage_order: app_config.testing.order.clone(),
        failure_policy: app_config.testing.failure_policy,
        real_delay_url: args
            .test_url
            .clone()
            .unwrap_or_else(|| app_config.testing.real_delay.url.clone()),
        accepted_http_statuses,
        follow_redirects: real_delay.follow_redirects,
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
        geoip_country_path,
        geoip_city_path,
        geoip_asn_path,
        geoip_lookup,
        gen_options,
    })
}
