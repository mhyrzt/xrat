use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::app::runtime_service::{ConnectRequest, RuntimeService};
use crate::tui::app::TuiApp;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

fn begin_runtime_op(
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) -> Option<(TuiTaskKind, bool, mpsc::UnboundedSender<TuiTaskEvent>)> {
    if app.task_state.running.is_some() {
        app.set_status("another operation is already running");
        return None;
    }
    let kind = TuiTaskKind::RuntimeOp;
    let include_deleted = app.config_list.include_deleted;
    let _ = app.task_state.start(kind);
    let _ = task_tx.send(TuiTaskEvent::Started { kind });
    Some((kind, include_deleted, task_tx.clone()))
}

async fn complete_after_reload(
    context: AppContext,
    include_deleted: bool,
    kind: TuiTaskKind,
    success_message: String,
) -> TuiTaskEvent {
    match TuiData::load(&context, include_deleted).await {
        Ok(data) => TuiTaskEvent::Completed {
            kind,
            message: success_message.clone(),
            data: Some(data),
        },
        Err(err) => TuiTaskEvent::Failed {
            kind,
            error: format!("{success_message} but reload failed: {err}"),
        },
    }
}

pub fn spawn_runtime_start(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let config_id = match app
        .data
        .runtime
        .selected_config_id
        .or(app.data.runtime.active_config_id)
    {
        Some(id) => id,
        None => {
            app.set_status("no selected config — select one from the Configs view first");
            return;
        }
    };
    let Some((kind, include_deleted, task_tx)) = begin_runtime_op(app, task_tx) else {
        return;
    };
    tokio::spawn(async move {
        let service = RuntimeService::new(&context);
        let event = match service.connect(ConnectRequest { config_id }).await {
            Ok(res) => {
                let msg = format!("started runtime with config #{}", res.config.id);
                complete_after_reload(context, include_deleted, kind, msg).await
            }
            Err(err) => TuiTaskEvent::Failed {
                kind,
                error: format!("runtime start failed: {err}"),
            },
        };
        let _ = task_tx.send(event);
    });
}

pub fn spawn_runtime_stop(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let Some((kind, include_deleted, task_tx)) = begin_runtime_op(app, task_tx) else {
        return;
    };
    tokio::spawn(async move {
        let service = RuntimeService::new(&context);
        let event = match service.disconnect().await {
            Ok(_) => {
                complete_after_reload(context, include_deleted, kind, "runtime stopped".to_string())
                    .await
            }
            Err(err) => TuiTaskEvent::Failed {
                kind,
                error: format!("runtime stop failed: {err}"),
            },
        };
        let _ = task_tx.send(event);
    });
}

pub fn spawn_runtime_restart(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let config_id = match app
        .data
        .runtime
        .active_config_id
        .or(app.data.runtime.selected_config_id)
    {
        Some(id) => id,
        None => {
            app.set_status("no active or selected config to restart with");
            return;
        }
    };
    let Some((kind, include_deleted, task_tx)) = begin_runtime_op(app, task_tx) else {
        return;
    };
    tokio::spawn(async move {
        let service = RuntimeService::new(&context);
        if let Err(err) = service.disconnect().await {
            let _ = task_tx.send(TuiTaskEvent::Failed {
                kind,
                error: format!("restart: stop failed: {err}"),
            });
            return;
        }
        let event = match service.connect(ConnectRequest { config_id }).await {
            Ok(res) => {
                let msg = format!("restarted runtime with config #{}", res.config.id);
                complete_after_reload(context, include_deleted, kind, msg).await
            }
            Err(err) => TuiTaskEvent::Failed {
                kind,
                error: format!("restart: start failed: {err}"),
            },
        };
        let _ = task_tx.send(event);
    });
}

pub fn spawn_runtime_switch(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let config_id = match app
        .data
        .runtime
        .selected_config_id
        .filter(|&id| Some(id) != app.data.runtime.active_config_id)
        .or(app.data.runtime.selected_config_id)
    {
        Some(id) => id,
        None => {
            app.set_status("no selected config — select one from the Configs view first");
            return;
        }
    };
    let Some((kind, include_deleted, task_tx)) = begin_runtime_op(app, task_tx) else {
        return;
    };
    tokio::spawn(async move {
        let service = RuntimeService::new(&context);
        if let Err(err) = service.disconnect().await {
            let _ = task_tx.send(TuiTaskEvent::Failed {
                kind,
                error: format!("switch: stop failed: {err}"),
            });
            return;
        }
        let event = match service.connect(ConnectRequest { config_id }).await {
            Ok(res) => {
                let msg = format!("switched runtime to config #{}", res.config.id);
                complete_after_reload(context, include_deleted, kind, msg).await
            }
            Err(err) => TuiTaskEvent::Failed {
                kind,
                error: format!("switch: start failed: {err}"),
            },
        };
        let _ = task_tx.send(event);
    });
}
