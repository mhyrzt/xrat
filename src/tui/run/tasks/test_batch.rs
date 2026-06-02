use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::app::{TestMode, TuiApp};
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

pub fn spawn_test_batch(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    if app.task_state.running.is_some() {
        app.set_status("another operation is already running");
        return;
    }

    let config_ids = app.test_config_ids();
    if config_ids.is_empty() {
        app.set_status("no configs match the current test scope");
        return;
    }

    let kind = TuiTaskKind::TestBatch;
    let args = test_args_for_app(app);
    let include_deleted = app.config_list.include_deleted;
    let (token, receiver) = app.task_state.start(kind);
    let _ = task_tx.send(TuiTaskEvent::Started { kind });

    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<(usize, usize)>();
    let task_tx_clone = task_tx.clone();
    tokio::spawn(async move {
        while let Some((done, total)) = progress_rx.recv().await {
            let _ = task_tx_clone.send(TuiTaskEvent::Progress { kind, done, total });
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
                },
            },
            Err(_) if was_cancelled => TuiTaskEvent::Cancelled { kind },
            Err(error) => TuiTaskEvent::Failed {
                kind,
                error: error.to_string(),
            },
        };
        let _ = task_tx.send(event);
    });
}

fn test_args_for_app(app: &TuiApp) -> crate::cli::TestArgs {
    let (skip_tcp, skip_real_delay, skip_download) = match app.test_state.mode {
        TestMode::Tcp => (false, false, true),
        TestMode::RealDelay => (true, false, true),
        TestMode::Both => (false, false, true),
    };

    crate::cli::TestArgs {
        id: None,
        enabled_only: false,
        active_only: false,
        selected_only: false,
        subscription: None,
        skip_icmp: true,
        skip_tcp,
        skip_real_delay,
        skip_download,
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
