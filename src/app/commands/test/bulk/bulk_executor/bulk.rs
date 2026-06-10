use super::progress::{
    bulk_progress_bar, finish_bulk_progress, spawn_config_test, update_bulk_progress,
};
use super::*;
use crate::support::cancel::CancellationReceiver;

pub(super) async fn run_bulk(
    args: &TestArgs,
    context: &AppContext,
    settings: ResolvedTestSettings,
    subscription_id: Option<i64>,
) -> crate::app::Result<()> {
    let configs = context
        .db
        .list_configs(&args.config_filter(subscription_id))
        .await?;
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
    run_bulk_for_configs_cancellable(
        context,
        settings,
        configs,
        run_kind,
        show_progress,
        None,
        None,
    )
    .await
}

pub(crate) async fn run_bulk_for_configs_cancellable(
    context: &AppContext,
    settings: ResolvedTestSettings,
    configs: Vec<ConfigRecord>,
    run_kind: &str,
    show_progress: bool,
    cancel_rx: Option<CancellationReceiver>,
    progress_tx: Option<tokio::sync::mpsc::UnboundedSender<TestProgressUpdate>>,
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
    if run_kind == "rotation" && total > 0 {
        crate::app::events::record(
            &context.db,
            crate::app::events::LEVEL_INFO,
            crate::app::events::SOURCE_ROTATION,
            "rotation_bulk_progress",
            "rotation_bulk_started".to_string(),
            None,
            None,
            Some(format!("{{\"done\":0,\"total\":{total}}}")),
        )
        .await;
    }
    let mut next_config = configs.into_iter();
    let mut join_set = JoinSet::new();
    let mut completed = 0usize;
    let mut failed = 0usize;
    let mut outputs = Vec::with_capacity(total);

    let cancelled = || cancel_rx.as_ref().is_some_and(|rx| rx.is_cancelled());

    for _ in 0..concurrency {
        if cancelled() {
            break;
        }
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
        match joined {
            Ok(Ok(output)) => {
                completed += 1;
                if output.status != TestStatus::Ok {
                    failed += 1;
                }
                let config_id = output.id;
                outputs.push(output);
                update_bulk_progress(&progress, completed, failed);
                if let Some(tx) = &progress_tx {
                    let _ = tx.send(TestProgressUpdate {
                        config_id,
                        done: completed,
                        total,
                    });
                }
            }
            Ok(Err(error)) => return Err(error),
            Err(join_error) if join_error.is_cancelled() => {}
            Err(join_error) => return Err(join_error.into()),
        }

        if cancelled() {
            join_set.abort_all();
            continue;
        }

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
    if run_kind == "rotation" {
        crate::app::events::record(
            &context.db,
            crate::app::events::LEVEL_INFO,
            crate::app::events::SOURCE_ROTATION,
            "rotation_bulk_progress",
            "rotation_bulk_finished".to_string(),
            None,
            None,
            Some(format!("{{\"done\":{completed},\"total\":{total}}}")),
        )
        .await;
    }

    let level = if failed > 0 && failed == completed {
        crate::app::events::LEVEL_WARN
    } else {
        crate::app::events::LEVEL_INFO
    };
    crate::app::events::record(
        &context.db,
        level,
        crate::app::events::SOURCE_TEST,
        "test_run",
        format!(
            "Tested {completed} config(s): {} passed, {failed} failed",
            completed.saturating_sub(failed)
        ),
        None,
        None,
        Some(format!("{{\"run_id\":{run_id},\"kind\":\"{run_kind}\"}}")),
    )
    .await;

    Ok(outputs)
}
