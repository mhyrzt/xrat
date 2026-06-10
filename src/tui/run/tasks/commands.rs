use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::app::{TuiApp, TuiConfigCommand};
use crate::tui::task::TuiTaskEvent;

/// Apply a single-config command. Enable/disable mutate the affected row in
/// place; commands that change which rows are visible (delete/restore/purge)
/// spawn a background reload instead of blocking the event loop on a full load.
pub async fn run_config_command(
    context: &AppContext,
    app: &mut TuiApp,
    command: TuiConfigCommand,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    match command {
        TuiConfigCommand::Enable(id) => apply_enabled(context, app, id, true).await,
        TuiConfigCommand::Disable(id) => apply_enabled(context, app, id, false).await,
        TuiConfigCommand::Restore(id) => {
            apply_reload(
                app,
                context.db.restore_config(id).await,
                id,
                "restored",
                task_tx,
                context,
            );
        }
        TuiConfigCommand::SoftDelete(id) => {
            apply_reload(
                app,
                context.db.delete_config(id).await,
                id,
                "soft deleted",
                task_tx,
                context,
            );
        }
        TuiConfigCommand::Purge(id) => {
            apply_reload(
                app,
                context.db.hard_delete_config(id).await,
                id,
                "purged",
                task_tx,
                context,
            );
        }
    }
}

async fn apply_enabled(context: &AppContext, app: &mut TuiApp, id: i64, enabled: bool) {
    match context.db.set_config_enabled(id, enabled).await {
        Ok(_) => {
            app.data.set_config_enabled(id, enabled);
            let verb = if enabled { "enabled" } else { "disabled" };
            app.push_log(format!("OK  {verb} config {id}"));
        }
        Err(error) => app.push_log(format!("ERR operation failed: {error}")),
    }
}

fn apply_reload(
    app: &mut TuiApp,
    result: crate::db::Result<()>,
    id: i64,
    verb: &str,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
    context: &AppContext,
) {
    match result {
        Ok(_) => {
            super::spawn_reload_data(context.clone(), app.config_list.include_deleted, task_tx);
            app.push_log(format!("OK  {verb} config {id}"));
        }
        Err(error) => app.push_log(format!("ERR operation failed: {error}")),
    }
}
