mod tasks;
mod terminal;

use std::time::Duration;

use crossterm::event::{self, Event};
use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::tui::app::TuiApp;
use crate::tui::data::TuiData;
use crate::tui::task::TuiTaskEvent;

use terminal::TerminalSession;

pub async fn run(context: &AppContext) -> crate::app::Result<()> {
    let mut terminal = TerminalSession::enter()?;
    let data = TuiData::load(context, false).await?;
    let mut app = TuiApp::with_data(data);
    let (task_tx, mut task_rx) = mpsc::unbounded_channel();

    loop {
        drain_task_events(&mut app, &mut task_rx);
        terminal.draw(|frame| crate::tui::view::render(frame, &app))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Resize(_, _) => {}
                Event::Key(key) => {
                    let action = crate::tui::keymap::action_for_key(
                        key,
                        app.active_view,
                        app.config_list.editing_search,
                        app.confirm.is_some(),
                        app.import_modal.is_some(),
                        app.rename_modal.is_some(),
                        app.qr_modal.is_some(),
                    );
                    let confirmed_command = if matches!(action, crate::tui::app::TuiAction::Confirm)
                    {
                        app.pending_confirm_command()
                    } else {
                        None
                    };
                    let confirmed_source_delete =
                        if matches!(action, crate::tui::app::TuiAction::Confirm) {
                            if let Some(confirm) = &app.confirm {
                                if let crate::tui::app::ConfirmKind::DeleteSource(id) =
                                    confirm.kind
                                {
                                    Some(id)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                    let direct_command = app.config_command_for_action(action);
                    let bulk_enable_ids: Vec<i64> =
                        if matches!(action, crate::tui::app::TuiAction::EnableSelected) {
                            app.data
                                .configs
                                .iter()
                                .filter(|c| c.is_selected && !c.is_deleted)
                                .map(|c| c.id)
                                .collect()
                        } else {
                            Vec::new()
                        };
                    let bulk_disable_ids: Vec<i64> =
                        if matches!(action, crate::tui::app::TuiAction::DisableSelected) {
                            app.data
                                .configs
                                .iter()
                                .filter(|c| c.is_selected && !c.is_deleted)
                                .map(|c| c.id)
                                .collect()
                        } else {
                            Vec::new()
                        };
                    let is_sources_view = app.active_view == crate::tui::app::TuiView::Sources;
                    let qr_config_id =
                        if matches!(action, crate::tui::app::TuiAction::OpenQrFocused)
                            && !is_sources_view
                        {
                            app.focused_config()
                                .map(|c| (c.id, c.display_name().to_string()))
                        } else {
                            None
                        };
                    let qr_source = if matches!(action, crate::tui::app::TuiAction::OpenQrFocused)
                        && is_sources_view
                    {
                        app.focused_source()
                            .filter(|s| !s.value.is_empty())
                            .map(|s| (s.display_name().to_string(), s.value.clone()))
                    } else {
                        None
                    };
                    let copy_focused_id =
                        if matches!(action, crate::tui::app::TuiAction::CopyFocused)
                            && !is_sources_view
                        {
                            app.focused_config().map(|c| c.id)
                        } else {
                            None
                        };
                    let copy_focused_source =
                        if matches!(action, crate::tui::app::TuiAction::CopyFocused)
                            && is_sources_view
                        {
                            app.focused_source()
                                .filter(|s| !s.value.is_empty())
                                .map(|s| s.value.clone())
                        } else {
                            None
                        };
                    let copy_selected_ids: Vec<i64> =
                        if matches!(action, crate::tui::app::TuiAction::CopySelected) {
                            app.data
                                .configs
                                .iter()
                                .filter(|c| c.is_selected)
                                .map(|c| c.id)
                                .collect()
                        } else {
                            Vec::new()
                        };
                    let import_input = if matches!(action, crate::tui::app::TuiAction::ImportSubmit)
                    {
                        app.import_modal
                            .as_ref()
                            .map(|m| m.input.trim().to_string())
                    } else {
                        None
                    };
                    let rename_submit =
                        if matches!(action, crate::tui::app::TuiAction::RenameSubmit) {
                            app.rename_modal
                                .as_ref()
                                .map(|m| (m.source_id, m.input.trim().to_string()))
                        } else {
                            None
                        };
                    let open_rename = matches!(action, crate::tui::app::TuiAction::OpenRenameModal);
                    let rename_prefill = if open_rename {
                        app.focused_source()
                            .map(|s| (s.id, s.name.clone().unwrap_or_default()))
                    } else {
                        None
                    };
                    let focused_source_value =
                        if matches!(action, crate::tui::app::TuiAction::RefreshFocusedSource) {
                            app.focused_source()
                                .map(|s| (s.id, s.value.clone()))
                                .filter(|(_, v)| !v.is_empty())
                        } else {
                            None
                        };
                    let all_source_values: Vec<(i64, String)> =
                        if matches!(action, crate::tui::app::TuiAction::RefreshAllSources) {
                            app.data
                                .sources
                                .iter()
                                .filter(|s| !s.value.is_empty())
                                .map(|s| (s.id, s.value.clone()))
                                .collect()
                        } else {
                            Vec::new()
                        };
                    app.apply(action);
                    if matches!(action, crate::tui::app::TuiAction::ToggleDeletedFilter) {
                        tasks::spawn_reload_data(
                            context.clone(),
                            app.config_list.include_deleted,
                            &task_tx,
                        );
                    }
                    if matches!(action, crate::tui::app::TuiAction::StartTestBatch) {
                        tasks::spawn_test_batch(context.clone(), &mut app, &task_tx);
                    }
                    if matches!(action, crate::tui::app::TuiAction::RuntimeStart) {
                        tasks::spawn_runtime_start(context.clone(), &mut app, &task_tx);
                    }
                    if matches!(action, crate::tui::app::TuiAction::RuntimeStop) {
                        tasks::spawn_runtime_stop(context.clone(), &mut app, &task_tx);
                    }
                    if matches!(action, crate::tui::app::TuiAction::RuntimeRestart) {
                        tasks::spawn_runtime_restart(context.clone(), &mut app, &task_tx);
                    }
                    if matches!(action, crate::tui::app::TuiAction::RuntimeSwitch) {
                        tasks::spawn_runtime_switch(context.clone(), &mut app, &task_tx);
                    }
                    if let Some((source_id, source_value)) = focused_source_value {
                        tasks::spawn_source_refresh(
                            context.clone(),
                            source_id,
                            source_value,
                            app.config_list.include_deleted,
                            &task_tx,
                        );
                    }
                    if !all_source_values.is_empty() {
                        tasks::spawn_source_refresh_all(
                            context.clone(),
                            all_source_values,
                            app.config_list.include_deleted,
                            &task_tx,
                        );
                    }
                    if let Some(input) = import_input {
                        if !input.is_empty() {
                            app.import_modal = None;
                            tasks::spawn_source_import(
                                context.clone(),
                                input,
                                app.config_list.include_deleted,
                                &task_tx,
                            );
                        } else if let Some(modal) = &mut app.import_modal {
                            modal.error = Some("input is empty".to_string());
                        }
                    }
                    if let Some((id, name)) = rename_prefill {
                        app.rename_modal = Some(crate::tui::app::RenameModalState {
                            source_id: id,
                            input: name,
                            error: None,
                        });
                    }
                    if let Some((source_id, name)) = rename_submit {
                        app.rename_modal = None;
                        tasks::run_source_rename(context, &mut app, source_id, name).await;
                    }
                    if let Some(source_id) = confirmed_source_delete {
                        tasks::run_source_delete(context, &mut app, source_id).await;
                    }
                    if let Some((config_id, config_name)) = qr_config_id {
                        tasks::open_qr_for_config(context, &mut app, config_id, config_name).await;
                    }
                    if let Some((source_name, source_url)) = qr_source {
                        tasks::open_qr_for_source(&mut app, source_name, source_url);
                    }
                    if let Some(config_id) = copy_focused_id {
                        tasks::copy_config_uri(context, &mut app, config_id).await;
                    }
                    if let Some(source_url) = copy_focused_source {
                        tasks::copy_source_uri(&mut app, source_url);
                    }
                    if matches!(action, crate::tui::app::TuiAction::OpenQrApiUrl) {
                        tasks::open_qr_for_api_url(&mut app);
                    }
                    if matches!(action, crate::tui::app::TuiAction::CopyApiUrl) {
                        tasks::copy_api_url(&mut app);
                    }
                    if !copy_selected_ids.is_empty() {
                        tasks::copy_selected_uris(context, &mut app, copy_selected_ids).await;
                    }
                    if !bulk_enable_ids.is_empty() {
                        tasks::run_bulk_enable_disable(context, &mut app, bulk_enable_ids, true)
                            .await;
                    }
                    if !bulk_disable_ids.is_empty() {
                        tasks::run_bulk_enable_disable(context, &mut app, bulk_disable_ids, false)
                            .await;
                    }
                    if let Some(command) = confirmed_command.or(direct_command) {
                        tasks::run_config_command(context, &mut app, command).await;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn drain_task_events(app: &mut TuiApp, task_rx: &mut mpsc::UnboundedReceiver<TuiTaskEvent>) {
    while let Ok(event) = task_rx.try_recv() {
        app.apply_task_event(event);
    }
}
