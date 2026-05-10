use super::*;

pub async fn run(args: &TestArgs, context: &AppContext) -> crate::app::Result<()> {
    if args.latest_run_summary {
        print_latest_run_summary(&context.db, args).await?;
        return Ok(());
    }

    let settings = resolve_test_settings(args, &context.app_config, &context.runtime_paths)?;

    if args.ping {
        return run_ping_loop(args, context, settings).await;
    }

    if let Some(config_id) = args.id {
        run_single(args, context, settings, config_id).await
    } else {
        run_bulk(args, context, settings).await
    }
}

pub(super) async fn run_ping_loop(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
) -> crate::app::Result<()> {
    let config_id = args.id.ok_or_else(|| {
        AppError::InvalidArgument(
            "`test --ping` requires config id: `xrat test <id> --ping`".into(),
        )
    })?;
    let config = context
        .db
        .get_config_by_id(config_id)
        .await?
        .ok_or_else(|| AppError::InvalidArgument(format!("config id {config_id} not found")))?;
    let run_id = context
        .db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "ping_loop".to_string(),
        })
        .await?;
    let interval_ms = args.ping_interval_ms.max(100);
    println!(
        "Starting ping loop for config #{} (interval={}ms). Press Ctrl+C to stop.",
        config_id, interval_ms
    );

    let mut ticker = tokio::time::interval(Duration::from_millis(interval_ms));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let started_at = Instant::now();
    let mut total = 0usize;
    let mut ok = 0usize;
    let mut failed = 0usize;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                break;
            }
            _ = ticker.tick() => {
                total += 1;
                let output = test_and_record_config(
                    context.db.clone(),
                    config.clone(),
                    settings.clone(),
                    false,
                    Some(run_id),
                ).await?;
                match output.status {
                    TestStatus::Ok => {
                        ok += 1;
                        println!(
                            "#{total} ok icmp={}ms real_delay={}ms download={}Mbps",
                            optional_number(output.icmp_ms),
                            optional_number(output.real_delay_ms),
                            optional_float(output.download_mbps),
                        );
                    }
                    TestStatus::Failed => {
                        failed += 1;
                        println!("#{total} failed {}", output.error.as_deref().unwrap_or("failed"));
                    }
                    TestStatus::Skipped => {
                        println!("#{total} skipped");
                    }
                }
            }
        }
    }

    let elapsed = started_at.elapsed().as_secs_f64();
    println!(
        "Ping loop stopped: total={}, ok={}, failed={}, elapsed={:.2}s, run_id={}",
        total, ok, failed, elapsed, run_id
    );
    Ok(())
}

pub(super) async fn print_latest_run_summary(
    db: &Database,
    args: &TestArgs,
) -> crate::app::Result<()> {
    let Some(run) = db.get_latest_connection_test_run().await? else {
        println!("No persisted test runs found.");
        return Ok(());
    };
    let tests = db.list_connection_tests_by_run(run.id).await?;
    let tests = filter_latest_run_rows(tests, args.country.as_deref(), args.asn.as_deref());
    let total = tests.len();
    let failed = tests
        .iter()
        .filter(|row| row.failure_kind.is_some())
        .count();
    let ok = total.saturating_sub(failed);
    println!(
        "Latest test run #{} ({}) at {}: total={}, ok={}, failed={}",
        run.id, run.kind, run.created_at, total, ok, failed
    );
    print_geo_distribution(
        "Country distribution",
        tests
            .iter()
            .filter_map(|row| row.endpoint_country.as_deref()),
    );
    print_geo_distribution(
        "ASN distribution",
        tests.iter().filter_map(|row| row.endpoint_asn.as_deref()),
    );
    Ok(())
}

pub(super) fn filter_latest_run_rows(
    rows: Vec<crate::db::ConnectionTestRecord>,
    country: Option<&str>,
    asn: Option<&str>,
) -> Vec<crate::db::ConnectionTestRecord> {
    let country = country
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_uppercase());
    let asn = asn
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());

    rows.into_iter()
        .filter(|row| {
            let country_match = country.as_ref().map_or(true, |filter| {
                row.endpoint_country
                    .as_deref()
                    .map(|value| value.eq_ignore_ascii_case(filter))
                    .unwrap_or(false)
            });
            let asn_match = asn.as_ref().map_or(true, |filter| {
                row.endpoint_asn
                    .as_deref()
                    .map(|value| value.to_ascii_lowercase().contains(filter))
                    .unwrap_or(false)
            });
            country_match && asn_match
        })
        .collect()
}
