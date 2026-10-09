use tokio::sync::mpsc;
use tracing::Instrument;

use crate::app::context::AppContext;
use crate::app::services::runtime_control;
use crate::tui::app::TuiApp;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

fn begin_runtime_op(
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) -> Option<(TuiTaskKind, bool, mpsc::UnboundedSender<TuiTaskEvent>)> {
    if app.task_state.running.is_some() {
        return None;
    }
    let kind = TuiTaskKind::RuntimeOp;
    let include_deleted = app.config_list.include_deleted;
    let _ = app.task_state.start(kind);
    if task_tx.send(TuiTaskEvent::Started { kind }).is_err() {
        tracing::debug!(?kind, "TUI task receiver dropped before runtime operation");
    }
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
            data: None,
        },
    }
}

async fn fail_after_reload(
    context: AppContext,
    include_deleted: bool,
    kind: TuiTaskKind,
    error: String,
) -> TuiTaskEvent {
    let data = match TuiData::load(&context, include_deleted).await {
        Ok(data) => Some(data),
        Err(reload_error) => {
            tracing::debug!(?kind, %reload_error, "TUI reload after runtime failure failed");
            None
        }
    };
    TuiTaskEvent::Failed { kind, error, data }
}

pub fn spawn_runtime_start_config(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
    config_id: xrat_model::ConfigId,
) {
    let Some((kind, include_deleted, task_tx)) = begin_runtime_op(app, task_tx) else {
        return;
    };
    tokio::spawn(
        async move {
            let control = match runtime_control::tui_control(&context).await {
                Ok(control) => control,
                Err(error) => {
                    if task_tx
                        .send(TuiTaskEvent::Failed {
                            kind,
                            error: error.to_string(),
                            data: None,
                        })
                        .is_err()
                    {
                        tracing::debug!(
                            ?kind,
                            config_id = config_id.0,
                            "TUI task receiver dropped after runtime control failure"
                        );
                    }
                    return;
                }
            };
            let event = match control.connect(config_id).await {
                Ok(res) => {
                    let msg = format!("started runtime with config #{}", res.config_id);
                    complete_after_reload(context, include_deleted, kind, msg).await
                }
                Err(err) => {
                    fail_after_reload(
                        context,
                        include_deleted,
                        kind,
                        format!("runtime start failed: {err}"),
                    )
                    .await
                }
            };
            if task_tx.send(event).is_err() {
                tracing::debug!(
                    ?kind,
                    config_id = config_id.0,
                    "TUI task receiver dropped after runtime start"
                );
            }
        }
        .instrument(tracing::debug_span!(
            "tui_runtime_start",
            config_id = config_id.0
        )),
    );
}

pub fn spawn_runtime_stop(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let Some((kind, include_deleted, task_tx)) = begin_runtime_op(app, task_tx) else {
        return;
    };
    tokio::spawn(
        async move {
            let control = match runtime_control::tui_control(&context).await {
                Ok(control) => control,
                Err(error) => {
                    if task_tx
                        .send(TuiTaskEvent::Failed {
                            kind,
                            error: error.to_string(),
                            data: None,
                        })
                        .is_err()
                    {
                        tracing::debug!(
                            ?kind,
                            "TUI task receiver dropped after runtime control failure"
                        );
                    }
                    return;
                }
            };
            let event = match control.disconnect().await {
                Ok(_) => {
                    complete_after_reload(
                        context,
                        include_deleted,
                        kind,
                        "runtime stopped".to_string(),
                    )
                    .await
                }
                Err(err) => TuiTaskEvent::Failed {
                    kind,
                    error: format!("runtime stop failed: {err}"),
                    data: None,
                },
            };
            if task_tx.send(event).is_err() {
                tracing::debug!(?kind, "TUI task receiver dropped after runtime stop");
            }
        }
        .instrument(tracing::debug_span!("tui_runtime_stop")),
    );
}

pub fn spawn_runtime_restart(
    context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let config_id = match app.data.runtime.active_config_id {
        Some(id) => id,
        None => return,
    };
    let Some((kind, include_deleted, task_tx)) = begin_runtime_op(app, task_tx) else {
        return;
    };
    tokio::spawn(
        async move {
            let control = match runtime_control::tui_control(&context).await {
                Ok(control) => control,
                Err(error) => {
                    if task_tx
                        .send(TuiTaskEvent::Failed {
                            kind,
                            error: error.to_string(),
                            data: None,
                        })
                        .is_err()
                    {
                        tracing::debug!(
                            ?kind,
                            config_id = config_id.0,
                            "TUI task receiver dropped after runtime control failure"
                        );
                    }
                    return;
                }
            };
            if let Err(err) = control.disconnect().await {
                if task_tx
                    .send(TuiTaskEvent::Failed {
                        kind,
                        error: format!("restart: stop failed: {err}"),
                        data: None,
                    })
                    .is_err()
                {
                    tracing::debug!(
                        ?kind,
                        config_id = config_id.0,
                        "TUI task receiver dropped after restart stop failure"
                    );
                }
                return;
            }
            let event = match control.connect(config_id).await {
                Ok(res) => {
                    let msg = format!("restarted runtime with config #{}", res.config_id);
                    complete_after_reload(context, include_deleted, kind, msg).await
                }
                Err(err) => {
                    fail_after_reload(
                        context,
                        include_deleted,
                        kind,
                        format!("restart: start failed: {err}"),
                    )
                    .await
                }
            };
            if task_tx.send(event).is_err() {
                tracing::debug!(
                    ?kind,
                    config_id = config_id.0,
                    "TUI task receiver dropped after runtime restart"
                );
            }
        }
        .instrument(tracing::debug_span!(
            "tui_runtime_restart",
            config_id = config_id.0
        )),
    );
}

pub fn spawn_runtime_tun(
    mut context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
    enabled: Option<bool>,
) {
    let Some((kind, include_deleted, task_tx)) = begin_runtime_op(app, task_tx) else {
        return;
    };
    tokio::spawn(async move {
        let event =
            match crate::app::services::tun_control::apply(&mut context, enabled, true).await {
                Ok(state) => {
                    complete_after_reload(
                        context,
                        include_deleted,
                        kind,
                        crate::app::services::tun_control::message(&state),
                    )
                    .await
                }
                Err(error) => {
                    fail_after_reload(
                        context,
                        include_deleted,
                        kind,
                        format!("TUN change failed: {error}"),
                    )
                    .await
                }
            };
        let _ = task_tx.send(event);
    });
}

pub fn spawn_runtime_tun_split(
    mut context: AppContext,
    app: &mut TuiApp,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
    enabled: bool,
    split_mode: crate::app::config::TunSplitMode,
    blacklist: Vec<String>,
    whitelist: Vec<String>,
) {
    let Some((kind, include_deleted, task_tx)) = begin_runtime_op(app, task_tx) else {
        return;
    };
    tokio::spawn(async move {
        let result = crate::app::services::tun_control::mutate_tun_settings(
            &mut context,
            true,
            move |tun| {
                tun.enabled = enabled;
                tun.split_mode = split_mode;
                tun.blacklist = blacklist;
                tun.whitelist = whitelist;
                Ok(())
            },
        )
        .await;
        let event = match result {
            Ok(state) => {
                complete_after_reload(
                    context,
                    include_deleted,
                    kind,
                    format!(
                        "split tunneling saved ({})",
                        crate::app::services::tun_control::split_summary(&state)
                    ),
                )
                .await
            }
            Err(error) => {
                fail_after_reload(
                    context,
                    include_deleted,
                    kind,
                    format!("split tunneling failed: {error}"),
                )
                .await
            }
        };
        let _ = task_tx.send(event);
    });
}
