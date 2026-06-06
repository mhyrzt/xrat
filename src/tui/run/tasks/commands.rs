use crate::app::context::AppContext;
use crate::tui::app::{TuiApp, TuiConfigCommand};

pub async fn run_config_command(context: &AppContext, app: &mut TuiApp, command: TuiConfigCommand) {
    let result = match command {
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
            super::data::reload_data(context, app).await;
            app.push_log(format!("OK  {message}"));
        }
        Err(error) => {
            let msg = format!("operation failed: {error}");
            app.push_log(format!("ERR {msg}"));
        }
    }
}
