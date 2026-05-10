use super::*;

pub(super) async fn test_and_record_config(
    db: Database,
    config: ConfigRecord,
    settings: ResolvedTestSettings,
    print_progress: bool,
    run_id: Option<i64>,
) -> crate::app::Result<TestOutputRow> {
    let node = node_from_record(&config)?;
    let mut result = TestResult::default();
    let test_start = Instant::now();
    let mut ran_icmp = false;
    let mut ran_tcp = false;
    let mut ran_real_delay = false;
    let mut ran_download = false;
    let mut ran_upload = false;

    for stage in &settings.stage_order {
        match stage {
            ConnectionTestStage::Icmp if settings.run_icmp => {
                ran_icmp = true;
                run_icmp_stage(&config, &settings, &mut result, print_progress).await?;
                if !result.icmp_ok && settings.failure_policy.halts_after_failure() {
                    break;
                }
            }
            ConnectionTestStage::RealDelay if settings.run_real_delay => {
                if settings.run_tcp && !ran_tcp {
                    ran_tcp = true;
                    run_tcp_gate(&config, &settings, &mut result, print_progress).await?;
                }

                if result.tcp_ok || !settings.run_tcp {
                    ran_real_delay = true;
                    run_real_delay_stage(&node, &settings, &mut result, print_progress).await?;
                    if !result.real_delay_ok && settings.failure_policy.halts_after_failure() {
                        break;
                    }
                } else if print_progress {
                    println!("Skipping real-delay test (TCP check failed)");
                }

                if settings.run_tcp
                    && !result.tcp_ok
                    && settings.failure_policy.halts_after_failure()
                {
                    break;
                }
            }
            ConnectionTestStage::Download if settings.run_download => {
                ran_download = true;
                run_download_stage(&node, &settings, &mut result, print_progress).await?;
                if settings.run_upload {
                    ran_upload = true;
                    run_upload_stage(&node, &settings, &mut result, print_progress).await?;
                }
                if !result.download_ok && settings.failure_policy.halts_after_failure() {
                    break;
                }
            }
            _ => {}
        }
    }

    let elapsed = test_start.elapsed();
    let output = TestOutputRow::from_parts(
        &config,
        &result,
        ran_icmp,
        ran_tcp,
        ran_real_delay,
        ran_download,
        ran_upload,
        elapsed,
    );
    db.insert_connection_test(&output.connection_test_insert(run_id))
        .await?;

    Ok(output)
}

pub(super) async fn run_icmp_stage(
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

pub(super) async fn run_tcp_gate(
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

pub(super) async fn run_real_delay_stage(
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
    )
    .await;
    let failure_reason = real_delay_result.failure_reason.clone();

    result.real_delay_ok = real_delay_result.success;
    result.real_delay_ms = real_delay_result.latency_ms;
    result.ttfb_ms = real_delay_result.ttfb_ms;
    result.http_status = real_delay_result.http_status;
    result.endpoint_ip = real_delay_result.endpoint_ip;
    let endpoint_meta = resolve_endpoint_meta(
        result.endpoint_ip.as_deref(),
        settings.geoip_enabled,
        &settings.geoip_city_path,
        &settings.geoip_country_path,
        &settings.geoip_asn_path,
    );
    result.endpoint_location = endpoint_meta.location;
    result.endpoint_country = endpoint_meta.country;
    result.endpoint_asn = endpoint_meta.asn;
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
