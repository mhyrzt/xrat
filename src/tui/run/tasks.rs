use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::app::{TestMode, TuiApp, TuiConfigCommand};
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

pub fn spawn_reload_data(
    context: AppContext,
    include_deleted: bool,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let kind = TuiTaskKind::ReloadData;
    let _ = task_tx.send(TuiTaskEvent::Started { kind });

    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let event = match TuiData::load(&context, include_deleted).await {
            Ok(data) => TuiTaskEvent::Completed {
                kind,
                message: "reloaded data".to_string(),
                data: Some(data),
            },
            Err(error) => TuiTaskEvent::Failed {
                kind,
                error: error.to_string(),
            },
        };
        let _ = task_tx.send(event);
    });
}

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

    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let result = crate::app::commands::test::run_bulk_for_config_ids_cancellable(
            &args,
            &context,
            &config_ids,
            receiver,
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

pub async fn run_config_command(context: &AppContext, app: &mut TuiApp, command: TuiConfigCommand) {
    let result = match command {
        TuiConfigCommand::Select(id) => context
            .db
            .set_selected_config(id)
            .await
            .map(|_| format!("selected config {id}")),
        TuiConfigCommand::Enable(id) => context
            .db
            .set_config_enabled(id, true)
            .await
            .map(|_| format!("enabled config {id}")),
        TuiConfigCommand::Disable(id) => context
            .db
            .set_config_enabled(id, false)
            .await
            .map(|_| format!("disabled config {id}")),
        TuiConfigCommand::Restore(id) => context
            .db
            .restore_config(id)
            .await
            .map(|_| format!("restored config {id}")),
        TuiConfigCommand::SoftDelete(id) => context
            .db
            .delete_config(id)
            .await
            .map(|_| format!("soft deleted config {id}")),
        TuiConfigCommand::Purge(id) => context
            .db
            .hard_delete_config(id)
            .await
            .map(|_| format!("purged config {id}")),
    };

    match result {
        Ok(message) => {
            reload_data(context, app).await;
            app.set_status(message);
        }
        Err(error) => app.set_status(format!("operation failed: {error}")),
    }
}

pub async fn reload_data(context: &AppContext, app: &mut TuiApp) {
    match TuiData::load(context, app.config_list.include_deleted).await {
        Ok(data) => app.reload_data(data),
        Err(error) => app.set_status(format!("reload failed: {error}")),
    }
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
