use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::app::TuiApp;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

pub fn spawn_test_batch(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    if app.task_state.running.is_some() {
        return;
    }

    let config_ids = app.test_config_ids();
    if config_ids.is_empty() {
        return;
    }
    app.data.clear_test_fields_for_configs(&config_ids);
    app.testing_config_ids = config_ids.clone();

    let kind = TuiTaskKind::TestBatch;
    let args = test_args_for_app(app);
    let include_deleted = app.config_list.include_deleted;
    let (token, receiver) = app.task_state.start(kind);
    let _ = task_tx.send(TuiTaskEvent::Started { kind });

    let (progress_tx, mut progress_rx) =
        mpsc::unbounded_channel::<crate::app::commands::test::TestProgressUpdate>();
    let task_tx_clone = task_tx.clone();
    let progress_context = context.clone();
    tokio::spawn(async move {
        while let Some(update) = progress_rx.recv().await {
            match progress_context
                .db
                .get_config_with_latest_test(update.config_id)
                .await
            {
                Ok(Some(row)) => {
                    let _ = task_tx_clone.send(TuiTaskEvent::ConfigTested {
                        row: row.into(),
                        done: update.done,
                        total: update.total,
                    });
                }
                _ => {
                    let _ = task_tx_clone.send(TuiTaskEvent::Progress {
                        kind,
                        done: update.done,
                        total: update.total,
                    });
                }
            }
        }
    });

    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let result = crate::app::commands::test::run_bulk_for_config_ids_with_progress(
            &args,
            &context,
            &config_ids,
            receiver,
            progress_tx,
        )
        .await;

        let was_cancelled = token.is_cancelled();
        let event = match result {
            Ok(_) if was_cancelled => TuiTaskEvent::Cancelled { kind },
            Ok(tested) => match TuiData::load(&context, include_deleted).await {
                Ok(data) => TuiTaskEvent::Completed {
                    kind,
                    message: format!("tested {tested} configs"),
                    data: Some(data),
                },
                Err(error) => TuiTaskEvent::Failed {
                    kind,
                    error: format!("test completed but reload failed: {error}"),
                    data: None,
                },
            },
            Err(_) if was_cancelled => TuiTaskEvent::Cancelled { kind },
            Err(error) => TuiTaskEvent::Failed {
                kind,
                error: error.to_string(),
                data: None,
            },
        };
        let _ = task_tx.send(event);
    });
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn test_args_for_app(app: &TuiApp) -> crate::cli::TestArgs {
    let run_icmp = app
        .data
        .test_stage_names
        .iter()
        .any(|stage| stage == "icmp");
    let run_real_delay = app
        .data
        .test_stage_names
        .iter()
        .any(|stage| stage == "real_delay");
    let run_download = app
        .data
        .test_stage_names
        .iter()
        .any(|stage| stage == "download");

    crate::cli::TestArgs {
        id: None,
        enabled_only: false,
        active_only: false,
        subscription: None,
        skip_icmp: !run_icmp,
        skip_tcp: true,
        skip_real_delay: !run_real_delay,
        skip_download: !run_download,
        skip_upload: true,
        test_url: None,
        download_url: None,
        upload_url: None,
        icmp_timeout_ms: None,
        tcp_timeout_ms: None,
        real_delay_timeout_ms: None,
        download_timeout_ms: None,
        upload_timeout_ms: None,
        concurrency: Some(app.test_state.concurrency as i32),
        format: crate::cli::TestFormat::default(),
        output: None,
        sort_by: crate::cli::TestSortBy::default(),
        no_progress: true,
        ping: false,
        ping_interval_ms: 1000,
        latest_run_summary: false,
        country: None,
        asn: None,
    }
}
