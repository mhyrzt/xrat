use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::app::TuiApp;
use crate::tui::data::{TuiData, TuiLogs};
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
                data: None,
            },
        };
        let _ = task_tx.send(event);
    });
}

pub async fn reload_data(context: &AppContext, app: &mut TuiApp) {
    match TuiData::load(context, app.config_list.include_deleted).await {
        Ok(data) => app.reload_data(data),
        Err(error) => app.push_log(format!("ERR reload failed: {error}")),
    }
}

pub fn spawn_reload_logs(
    context: AppContext,
    logs_tx: &mpsc::UnboundedSender<crate::app::Result<TuiLogs>>,
) {
    let logs_tx = logs_tx.clone();
    tokio::spawn(async move {
        let _ = logs_tx.send(TuiLogs::load(&context).await);
    });
}

async fn reload_logs(context: &AppContext, app: &mut TuiApp) {
    match TuiLogs::load(context).await {
        Ok(logs) => app.reload_logs(logs),
        Err(error) => app.push_log(format!("ERR log reload failed: {error}")),
    }
}

/// Clear all persisted events from the database, then refresh the logs card.
/// This is the destructive DB clear; engine log files are left untouched.
pub async fn run_clear_events(context: &AppContext, app: &mut TuiApp) {
    match context.db.clear_events().await {
        Ok(count) => {
            reload_logs(context, app).await;
            app.push_log(format!("OK  cleared {count} persisted event(s) from db"));
        }
        Err(error) => app.push_log(format!("ERR clear events failed: {error}")),
    }
}
