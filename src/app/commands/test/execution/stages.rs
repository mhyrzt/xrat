use super::*;

pub(crate) async fn run_icmp_stage(
    config: &ConfigRecord,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    if print_progress {
        print!("Running ICMP ping... ");
        std::io::stdout().flush()?;
    }
    let icmp_result = icmp_ping(&config.address, settings.icmp_timeout).await;
    let failure_reason = icmp_result.failure_reason.clone();

    result.icmp_ok = icmp_result.success;
    result.icmp_ms = icmp_result.latency_ms;
    merge_failure(result, icmp_result.failure_kind, icmp_result.failure_reason);

    if print_progress {
        print_stage_result(
            icmp_result.success,
            icmp_result.latency_ms,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}

pub(crate) async fn run_tcp_gate(
    config: &ConfigRecord,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    if print_progress {
        print!("Running TCP check... ");
        std::io::stdout().flush()?;
    }
    let tcp_result = tcp_check(&config.address, config.port as u16, settings.tcp_timeout).await;
    let failure_reason = tcp_result.failure_reason.clone();

    result.tcp_ok = tcp_result.success;
    result.tcp_ms = tcp_result.latency_ms;
    merge_failure(result, tcp_result.failure_kind, tcp_result.failure_reason);

    if print_progress {
        print_stage_result(
            tcp_result.success,
            tcp_result.latency_ms,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}

pub(crate) async fn run_real_delay_stage(
    node: &Node,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    if print_progress {
        print!("Running real-delay test... ");
        std::io::stdout().flush()?;
    }

    let real_delay_result = real_delay_check(
        node,
        &settings.real_delay_url,
        &settings.xray_binary_path,
        settings.xray_startup_timeout,
        settings.real_delay_timeout,
        &settings.gen_options,
        &settings.accepted_http_statuses,
        settings.follow_redirects,
    )
    .await;
    let failure_reason = real_delay_result.failure_reason.clone();

    result.real_delay_ok = real_delay_result.success;
    result.real_delay_ms = real_delay_result.latency_ms;
    result.ttfb_ms = real_delay_result.ttfb_ms;
    result.http_status = real_delay_result.http_status;
    result.dial_endpoint_ip = real_delay_result.dial_endpoint_ip;
    merge_failure(
        result,
        real_delay_result.failure_kind,
        real_delay_result.failure_reason,
    );

    if print_progress {
        print_stage_result(
            real_delay_result.success,
            real_delay_result.latency_ms,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}

/// Resolve dial-endpoint GeoIP metadata independently of the probe stages so it
/// is recorded per config (and streamed to the TUI) even when real-delay does
/// not run. Uses the configured dial address, not the probe-observed IP.
pub(crate) async fn run_geoip_stage(
    node: &Node,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
) {
    if !settings.geoip_enabled {
        return;
    }
    let endpoint_meta = resolve_endpoint_meta(
        Some(node.address.as_str()),
        settings.geoip_enabled,
        settings.geoip_lookup.as_ref(),
    )
    .await;
    result.dial_endpoint_location = endpoint_meta.location;
    result.dial_endpoint_country = endpoint_meta.country;
    result.dial_endpoint_asn = endpoint_meta.asn;
    result.dial_endpoint_geoip_source = endpoint_meta.source;
    result.dial_endpoint_fronting = endpoint_meta.fronting;
}
