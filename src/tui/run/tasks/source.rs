use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::app::import;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

pub fn spawn_source_refresh(
    context: AppContext,
    source_id: i64,
    source_value: String,
    include_deleted: bool,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let kind = TuiTaskKind::SourceRefresh;
    let _ = task_tx.send(TuiTaskEvent::Started { kind });
    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let event = refresh_one(&context, source_id, &source_value, include_deleted).await;
        let _ = task_tx.send(event);
    });
}

pub fn spawn_source_refresh_all(
    context: AppContext,
    sources: Vec<(i64, String)>,
    include_deleted: bool,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let kind = TuiTaskKind::SourceRefresh;
    let _ = task_tx.send(TuiTaskEvent::Started { kind });
    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let total = sources.len();
        let mut ok = 0usize;
        let mut removed_total = 0u64;
        let mut errors: Vec<String> = Vec::new();
        for (id, value) in &sources {
            match import_from(&context, value).await {
                Ok(summary) => {
                    tracing::debug!(
                        source_id = id,
                        imported = summary.imported_configs,
                        removed = summary.removed_configs,
                        "source refreshed"
                    );
                    removed_total += summary.removed_configs;
                    ok += 1;
                }
                Err(err) => {
                    errors.push(format!("#{id}: {err}"));
                }
            }
        }
        let event = if errors.is_empty() {
            match TuiData::load(&context, include_deleted).await {
                Ok(data) => TuiTaskEvent::Completed {
                    kind,
                    message: format!("refreshed {ok}/{total} sources ({removed_total} removed)"),
                    data: Some(data),
                },
                Err(err) => TuiTaskEvent::Failed {
                    kind,
                    error: format!("refresh done but reload failed: {err}"),
                    data: None,
                },
            }
        } else {
            TuiTaskEvent::Failed {
                kind,
                error: format!(
                    "{} of {total} sources failed: {}",
                    errors.len(),
                    errors.join("; ")
                ),
                data: None,
            }
        };
        let _ = task_tx.send(event);
    });
}

async fn refresh_one(
    context: &AppContext,
    source_id: i64,
    source_value: &str,
    include_deleted: bool,
) -> TuiTaskEvent {
    let kind = TuiTaskKind::SourceRefresh;
    match import_from(context, source_value).await {
        Ok(summary) => match TuiData::load(context, include_deleted).await {
            Ok(data) => TuiTaskEvent::Completed {
                kind,
                message: format!(
                    "refreshed source #{source_id}: {} imported, {} removed",
                    summary.imported_configs, summary.removed_configs
                ),
                data: Some(data),
            },
            Err(err) => TuiTaskEvent::Failed {
                kind,
                error: format!("refresh done but reload failed: {err}"),
                data: None,
            },
        },
        Err(err) => TuiTaskEvent::Failed {
            kind,
            error: format!("refresh source #{source_id} failed: {err}"),
            data: None,
        },
    }
}

pub async fn run_source_delete(
    context: &AppContext,
    app: &mut crate::tui::app::TuiApp,
    source_id: i64,
) {
    match context.db.delete_subscription_with_configs(source_id).await {
        Ok(_) => {
            super::data::reload_data(context, app).await;
            app.set_status(format!("deleted source #{source_id} and its configs"));
        }
        Err(err) => {
            app.set_status(format!("delete failed: {err}"));
        }
    }
}

pub async fn run_source_rename(
    context: &AppContext,
    app: &mut crate::tui::app::TuiApp,
    source_id: i64,
    name: String,
) {
    match context.db.set_subscription_name(source_id, &name).await {
        Ok(_) => {
            super::data::reload_data(context, app).await;
            app.set_status(format!("renamed source #{source_id}"));
        }
        Err(err) => {
            app.set_status(format!("rename failed: {err}"));
        }
    }
}

async fn import_from(
    context: &AppContext,
    input: &str,
) -> crate::app::Result<crate::db::ImportSummary> {
    let input = input.to_string();
    let (source, nodes) = tokio::task::spawn_blocking(move || import::load_nodes(&input)).await??;
    Ok(context.db.import_nodes(&source, &nodes).await?)
}
