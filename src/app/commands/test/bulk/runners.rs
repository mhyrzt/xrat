use super::super::*;

pub(super) async fn run_single(
    _args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    config_id: i64,
) -> crate::app::Result<()> {
    let config = context.db.get_config_by_id(config_id).await?;

    let Some(config) = config else {
        tracing::warn!(config_id, "config not found");
        return Ok(());
    };

    let run_id = context
        .db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: "single".to_string(),
        })
        .await?;

    print_single_header(&config);
    let output =
        test_and_record_config(context.db.clone(), config, settings, true, Some(run_id)).await?;
    print_single_summary(&output);

    Ok(())
}

pub(super) async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
) -> crate::app::Result<()> {
    let configs = context.db.list_configs(&args.config_filter()).await?;

    if configs.is_empty() {
        tracing::info!("no configs found for requested test filters");
        write_results(args, &[])?;
        return Ok(());
    }
    let mut outputs =
        run_bulk_for_configs(context, settings, configs, "bulk", !args.no_progress).await?;

    sort_results(&mut outputs, args.sort_by);
    write_results(args, &outputs)?;

    Ok(())
}

pub(crate) async fn run_bulk_for_configs(
    context: &AppContext,
    settings: ResolvedTestSettings,
    configs: Vec<ConfigRecord>,
    run_kind: &str,
    show_progress: bool,
) -> crate::app::Result<Vec<TestOutputRow>> {
    let run_id = context
        .db
        .insert_connection_test_run(&ConnectionTestRunInsert {
            kind: run_kind.to_string(),
        })
        .await?;

    let total = configs.len();
    let concurrency = resolve_concurrency(settings.concurrency)?;
    let progress = bulk_progress_bar(total, show_progress);
    let mut next_config = configs.into_iter();
    let mut join_set = JoinSet::new();
    let mut completed = 0usize;
    let mut failed = 0usize;
    let mut outputs = Vec::with_capacity(total);

    for _ in 0..concurrency {
        let Some(config) = next_config.next() else {
            break;
        };
        spawn_config_test(
            &mut join_set,
            context.db.clone(),
            config,
            settings.clone(),
            Some(run_id),
        );
    }

    while let Some(joined) = join_set.join_next().await {
        let output = joined??;
        completed += 1;
        if output.status != TestStatus::Ok {
            failed += 1;
        }
        outputs.push(output);
        update_bulk_progress(&progress, completed, failed);

        if let Some(config) = next_config.next() {
            spawn_config_test(
                &mut join_set,
                context.db.clone(),
                config,
                settings.clone(),
                Some(run_id),
            );
        }
    }
    finish_bulk_progress(progress, completed, failed);
    Ok(outputs)
}

pub(super) fn bulk_progress_bar(total: usize, enabled: bool) -> Option<ProgressBar> {
    if !enabled {
        return None;
    }

    let progress = ProgressBar::new(total as u64);
    let style = ProgressStyle::with_template(
        "{spinner:.green} testing [{bar:32.cyan/blue}] {pos}/{len} failed:{msg}",
    )
    .unwrap_or_else(|_| ProgressStyle::default_bar())
    .progress_chars("=>-");
    progress.set_style(style);
    progress.set_message("0");

    Some(progress)
}

pub(super) fn update_bulk_progress(
    progress: &Option<ProgressBar>,
    completed: usize,
    failed: usize,
) {
    if let Some(progress) = progress {
        progress.set_position(completed as u64);
        progress.set_message(failed.to_string());
    }
}

pub(super) fn finish_bulk_progress(progress: Option<ProgressBar>, completed: usize, failed: usize) {
    if let Some(progress) = progress {
        progress.finish_with_message(format!("done: {completed} tested, {failed} failed"));
    }
}

pub(super) fn spawn_config_test(
    join_set: &mut JoinSet<crate::app::Result<TestOutputRow>>,
    db: Database,
    config: ConfigRecord,
    settings: ResolvedTestSettings,
    run_id: Option<i64>,
) {
    join_set
        .spawn(async move { test_and_record_config(db, config, settings, false, run_id).await });
}
