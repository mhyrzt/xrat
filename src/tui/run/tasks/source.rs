use tokio::sync::mpsc;

use crate::app::commands::update::update_by_ids;
use crate::app::context::AppContext;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

pub fn spawn_source_refresh(
    context: AppContext,
    source_id: i64,
    include_deleted: bool,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let kind = TuiTaskKind::SourceRefresh;
    let _ = task_tx.send(TuiTaskEvent::Started { kind });
    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let event = refresh_one(&context, source_id, include_deleted).await;
        let _ = task_tx.send(event);
    });
}

pub fn spawn_source_refresh_all(
    context: AppContext,
    source_ids: Vec<i64>,
    include_deleted: bool,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let kind = TuiTaskKind::SourceRefresh;
    let _ = task_tx.send(TuiTaskEvent::Started { kind });
    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let event = match update_by_ids(&context, &source_ids).await {
            Ok(summary) if summary.failed == 0 => {
                match TuiData::load(&context, include_deleted).await {
                    Ok(data) => TuiTaskEvent::Completed {
                        kind,
                        message: summary.status_message(),
                        data: Some(data),
                    },
                    Err(err) => TuiTaskEvent::Failed {
                        kind,
                        error: format!("refresh done but reload failed: {err}"),
                        data: None,
                    },
                }
            }
            Ok(summary) => TuiTaskEvent::Failed {
                kind,
                error: summary.status_message(),
                data: None,
            },
            Err(error) => TuiTaskEvent::Failed {
                kind,
                error: format!("refresh subscriptions failed: {error}"),
                data: None,
            },
        };
        let _ = task_tx.send(event);
    });
}

async fn refresh_one(context: &AppContext, source_id: i64, include_deleted: bool) -> TuiTaskEvent {
    let kind = TuiTaskKind::SourceRefresh;
    match update_by_ids(context, &[source_id]).await {
        Ok(summary) if summary.failed == 0 => match TuiData::load(context, include_deleted).await {
            Ok(data) => TuiTaskEvent::Completed {
                kind,
                message: summary.status_message(),
                data: Some(data),
            },
            Err(err) => TuiTaskEvent::Failed {
                kind,
                error: format!("refresh done but reload failed: {err}"),
                data: None,
            },
        },
        Ok(summary) => TuiTaskEvent::Failed {
            kind,
            error: summary.status_message(),
            data: None,
        },
        Err(error) => TuiTaskEvent::Failed {
            kind,
            error: format!("refresh subscription #{source_id} failed: {error}"),
            data: None,
        },
    }
}

pub async fn run_source_delete(
    context: &AppContext,
    app: &mut crate::tui::app::TuiApp,
    source_id: i64,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    match context.db.delete_subscription_with_configs(source_id).await {
        Ok(_) => {
            super::spawn_reload_data(context.clone(), app.config_list.include_deleted, task_tx);
            app.push_log(format!(
                "OK  deleted subscription #{source_id} and its configs"
            ));
        }
        Err(err) => {
            app.push_log(format!("ERR delete failed: {err}"));
        }
    }
}

pub async fn run_source_rename(
    context: &AppContext,
    app: &mut crate::tui::app::TuiApp,
    source_id: i64,
    name: String,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    match context.db.set_subscription_name(source_id, &name).await {
        Ok(_) => {
            super::spawn_reload_data(context.clone(), app.config_list.include_deleted, task_tx);
            app.push_log(format!("OK  renamed subscription #{source_id}"));
        }
        Err(err) => {
            app.push_log(format!("ERR rename failed: {err}"));
        }
    }
}
