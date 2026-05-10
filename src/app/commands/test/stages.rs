use super::*;

pub(super) fn classify_endpoint_location(endpoint_ip: Option<&str>) -> Option<String> {
    let ip = endpoint_ip?.parse::<IpAddr>().ok()?;
    let label = match ip {
        IpAddr::V4(v4) => {
            if v4.is_private() {
                "private_ipv4"
            } else if v4.is_loopback() {
                "loopback_ipv4"
            } else if v4.is_link_local() {
                "link_local_ipv4"
            } else {
                "public"
            }
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() {
                "loopback_ipv6"
            } else if v6.is_unique_local() {
                "unique_local_ipv6"
            } else if v6.is_unicast_link_local() {
                "link_local_ipv6"
            } else {
                "public"
            }
        }
    };
    Some(label.to_string())
}

pub(super) struct EndpointMeta {
    pub(super) location: Option<String>,
    pub(super) country: Option<String>,
    pub(super) asn: Option<String>,
}

pub(super) fn resolve_endpoint_meta(
    endpoint_ip: Option<&str>,
    geoip_enabled: bool,
    geoip_city_path: &std::path::Path,
    geoip_country_path: &std::path::Path,
    geoip_asn_path: &std::path::Path,
) -> EndpointMeta {
    if geoip_enabled {
        if let Some(ip) = endpoint_ip {
            if let Some(city) = geoip::lookup_city_label(geoip_city_path, ip) {
                let country = city.split('/').next().map(str::to_string);
                return EndpointMeta {
                    location: Some(city),
                    country,
                    asn: geoip::lookup_asn_label(geoip_asn_path, ip),
                };
            }
            if let Some(country) = geoip::lookup_country_iso(geoip_country_path, ip) {
                return EndpointMeta {
                    location: Some(country.clone()),
                    country: Some(country),
                    asn: geoip::lookup_asn_label(geoip_asn_path, ip),
                };
            }
            if let Some(asn) = geoip::lookup_asn_label(geoip_asn_path, ip) {
                return EndpointMeta {
                    location: Some(asn.clone()),
                    country: None,
                    asn: Some(asn),
                };
            }
        }
    }
    EndpointMeta {
        location: classify_endpoint_location(endpoint_ip),
        country: None,
        asn: None,
    }
}

pub(super) async fn run_download_stage(
    node: &Node,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    if print_progress {
        print!("Running download speed test... ");
        std::io::stdout().flush()?;
    }

    let download_result = download_speed_check(
        node,
        &settings.download_url,
        &settings.xray_binary_path,
        settings.xray_startup_timeout,
        settings.download_timeout,
    )
    .await;
    let failure_reason = download_result.failure_reason.clone();

    result.download_ok = download_result.success;
    result.download_mbps = download_result.mbps;
    merge_failure(
        result,
        download_result.failure_kind,
        download_result.failure_reason,
    );

    if print_progress {
        print_download_result(
            download_result.success,
            download_result.mbps,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}

pub(super) async fn run_upload_stage(
    node: &Node,
    settings: &ResolvedTestSettings,
    result: &mut TestResult,
    print_progress: bool,
) -> crate::app::Result<()> {
    let Some(upload_url) = settings.upload_url.as_deref() else {
        return Ok(());
    };

    if print_progress {
        print!("Running upload speed test... ");
        std::io::stdout().flush()?;
    }

    let upload_result = upload_speed_check(
        node,
        upload_url,
        &settings.xray_binary_path,
        settings.xray_startup_timeout,
        settings.upload_timeout,
        settings.upload_payload_bytes,
    )
    .await;
    let failure_reason = upload_result.failure_reason.clone();

    result.upload_ok = upload_result.success;
    result.upload_mbps = upload_result.mbps;
    merge_failure(
        result,
        upload_result.failure_kind,
        upload_result.failure_reason,
    );

    if print_progress {
        print_download_result(
            upload_result.success,
            upload_result.mbps,
            failure_reason.as_deref(),
        );
    }

    Ok(())
}

pub(super) fn merge_failure(
    result: &mut TestResult,
    failure_kind: Option<FailureKind>,
    failure_reason: Option<String>,
) {
    if !matches!(result.failure_kind, None) {
        return;
    }

    result.failure_kind = failure_kind;
    result.failure_reason = failure_reason;
}

pub(super) fn print_download_result(
    success: bool,
    mbps: Option<f64>,
    failure_reason: Option<&str>,
) {
    if success {
        println!("OK {:.2} Mbps", mbps.unwrap_or_default());
    } else {
        println!("FAIL {}", failure_reason.unwrap_or("failed"));
    }
}

pub(super) fn print_stage_result(
    success: bool,
    latency_ms: Option<u32>,
    failure_reason: Option<&str>,
) {
    if success {
        println!("OK {}ms", latency_ms.unwrap_or_default());
    } else {
        println!("FAIL {}", failure_reason.unwrap_or("failed"));
    }
}
