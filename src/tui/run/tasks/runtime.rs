use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::app::runtime_service::{ConnectRequest, RuntimeService};
use crate::tui::app::TuiApp;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

pub fn spawn_runtime_start(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    if app.task_state.running.is_some() {
        app.set_status("another operation is already running");
        return;
    }

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

    let kind = TuiTaskKind::RuntimeOp;
    let include_deleted = app.config_list.include_deleted;
    let _ = app.task_state.start(kind);
    let _ = task_tx.send(TuiTaskEvent::Started { kind });

    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let service = RuntimeService::new(&context);
        let result = service.connect(ConnectRequest { config_id }).await;
        let event = match result {
            Ok(res) => match TuiData::load(&context, include_deleted).await {
                Ok(data) => TuiTaskEvent::Completed {
                    kind,
                    message: format!("started runtime with config #{}", res.config.id),
                    data: Some(data),
                },
                Err(err) => TuiTaskEvent::Failed {
                    kind,
                    error: format!("runtime started but reload failed: {err}"),
                },
            },
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
    if app.task_state.running.is_some() {
        app.set_status("another operation is already running");
        return;
    }

    let kind = TuiTaskKind::RuntimeOp;
    let include_deleted = app.config_list.include_deleted;
    let _ = app.task_state.start(kind);
    let _ = task_tx.send(TuiTaskEvent::Started { kind });

    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let service = RuntimeService::new(&context);
        let result = service.disconnect().await;
        let event = match result {
            Ok(_) => match TuiData::load(&context, include_deleted).await {
                Ok(data) => TuiTaskEvent::Completed {
                    kind,
                    message: "runtime stopped".to_string(),
                    data: Some(data),
                },
                Err(err) => TuiTaskEvent::Failed {
                    kind,
                    error: format!("runtime stopped but reload failed: {err}"),
                },
            },
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
    if app.task_state.running.is_some() {
        app.set_status("another operation is already running");
        return;
    }

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

    let kind = TuiTaskKind::RuntimeOp;
    let include_deleted = app.config_list.include_deleted;
    let _ = app.task_state.start(kind);
    let _ = task_tx.send(TuiTaskEvent::Started { kind });

    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let service = RuntimeService::new(&context);
        let stop_result = service.disconnect().await;
        if let Err(err) = stop_result {
            let _ = task_tx.send(TuiTaskEvent::Failed {
                kind,
                error: format!("restart: stop failed: {err}"),
            });
            return;
        }
        let start_result = service.connect(ConnectRequest { config_id }).await;
        let event = match start_result {
            Ok(res) => match TuiData::load(&context, include_deleted).await {
                Ok(data) => TuiTaskEvent::Completed {
                    kind,
                    message: format!("restarted runtime with config #{}", res.config.id),
                    data: Some(data),
                },
                Err(err) => TuiTaskEvent::Failed {
                    kind,
                    error: format!("runtime restarted but reload failed: {err}"),
                },
            },
            Err(err) => TuiTaskEvent::Failed {
                kind,
                error: format!("restart: start failed: {err}"),
            },
        };
        let _ = task_tx.send(event);
    });
}
