use crate::app::context::AppContext;
use crate::tui::app::{BulkKind, BulkOp, TuiApp};

pub async fn run_bulk_op(context: &AppContext, app: &mut TuiApp, op: BulkOp) {
    let ids = app.bulk_config_ids(op);
    if ids.is_empty() {
        return;
    }

    let result = match op.kind() {
        BulkKind::SoftDelete => context.db.delete_configs(&ids).await,
        BulkKind::Purge => context.db.hard_delete_configs(&ids).await,
        BulkKind::Restore => context.db.restore_configs(&ids).await,
    };

    match result {
        Ok(affected) => {
            super::data::reload_data(context, app).await;
            let message = format!("{} {affected} {} configs", op.verb(), op.target());
            app.push_log(format!("OK  {message}"));
        }
        Err(error) => {
            let msg = format!("operation failed: {error}");
            app.push_log(format!("ERR {msg}"));
        }
    }
}
