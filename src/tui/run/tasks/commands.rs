use crate::app::context::AppContext;
use crate::tui::app::{TuiApp, TuiConfigCommand};

pub async fn run_bulk_enable_disable(
    context: &AppContext,
    app: &mut TuiApp,
    ids: Vec<i64>,
    enable: bool,
) {
    let verb = if enable { "enabled" } else { "disabled" };
    let total = ids.len();
    let mut ok = 0usize;
    for id in ids {
        match context.db.set_config_enabled(id, enable).await {
            Ok(_) => ok += 1,
            Err(err) => {
                app.set_status(format!("bulk op failed at config {id}: {err}"));
                super::data::reload_data(context, app).await;
                return;
            }
        }
    }
    super::data::reload_data(context, app).await;
    app.set_status(format!("{verb} {ok}/{total} selected configs"));
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
            super::data::reload_data(context, app).await;
            app.push_log(format!("OK  {message}"));
            app.set_status(message);
        }
        Err(error) => {
            let msg = format!("operation failed: {error}");
            app.push_log(format!("ERR {msg}"));
            app.set_status(msg);
        }
    }
}
