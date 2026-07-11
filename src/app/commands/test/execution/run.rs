use super::*;

pub(crate) async fn test_and_record_config(
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
            ConnectionTestStage::Tcp if settings.run_tcp && !ran_tcp => {
                ran_tcp = true;
                run_tcp_gate(&config, &settings, &mut result, print_progress).await?;
                if !result.tcp_ok && settings.failure_policy.halts_after_failure() {
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

    run_geoip_stage(&node, &settings, &mut result).await;

    let elapsed = test_start.elapsed();
    let output = TestOutputRow::from_parts(TestOutputParts {
        config: &config,
        result: &result,
        ran_icmp,
        ran_tcp,
        ran_real_delay,
        ran_download,
        ran_upload,
        elapsed,
    });
    db.insert_connection_test(&output.connection_test_insert(run_id))
        .await?;

    Ok(output)
}
