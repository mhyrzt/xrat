use super::progress::{
    bulk_progress_bar, finish_bulk_progress, spawn_config_test, update_bulk_progress,
};
use super::*;

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
