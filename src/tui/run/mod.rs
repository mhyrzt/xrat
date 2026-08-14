mod tasks;
mod terminal;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use tasks::test_args_for_app;

use std::collections::BTreeSet;
use std::time::Duration;

use crossterm::event::{self, Event};
use tokio::sync::mpsc;

use crate::app::config::{ConfigEditSession, SettingEffect};
use crate::app::context::AppContext;
use crate::tui::app::{ConfirmKind, ConfirmState, SettingsModalState, TuiApp};
use crate::tui::data::TuiData;
use crate::tui::task::TuiTaskEvent;

use terminal::TerminalSession;

pub async fn run(context: &AppContext) -> crate::app::Result<()> {
    let mut context = context.clone();
    let mut terminal = TerminalSession::enter()?;
    let data = TuiData::load(&context, false).await?;
    let mut app = TuiApp::with_data(data);
    let (task_tx, mut task_rx) = mpsc::unbounded_channel();
    let (version_tx, mut version_rx) = mpsc::unbounded_channel();
    let (logs_tx, mut logs_rx) = mpsc::unbounded_channel();
    let (engines_tx, mut engines_rx) = mpsc::unbounded_channel();
    let (stats_tx, mut stats_rx) = mpsc::unbounded_channel();
    tasks::spawn_version_check(version_tx);
    tasks::spawn_probe_engines(context.clone(), &engines_tx);
    let mut geo_lookup =
        crate::tui::data::build_geo_lookup(&context.app_config, &context.runtime_paths);
    tasks::spawn_enrich_locations(
        context.db.clone(),
        geo_lookup.clone(),
        tasks::enrichment_targets(&app.data),
        &task_tx,
    );
    let mut last_log_refresh = std::time::Instant::now();
    let mut log_refresh_pending = false;
    let mut last_stats_poll = std::time::Instant::now();
    let mut stats_poll_pending = false;
    let mut needs_redraw = true;

    loop {
        let drain = drain_task_events(&mut app, &mut task_rx);
        needs_redraw |= drain.any;
        if drain.reloaded {
            tasks::spawn_enrich_locations(
                context.db.clone(),
                geo_lookup.clone(),
                tasks::enrichment_targets(&app.data),
                &task_tx,
            );
        }
        while let Ok(result) = logs_rx.try_recv() {
            log_refresh_pending = false;
            needs_redraw = true;
            match result {
                Ok(logs) => app.reload_logs(logs),
                Err(error) => app.push_log(format!("ERR log refresh failed: {error}")),
            }
        }
        if !log_refresh_pending && last_log_refresh.elapsed() >= Duration::from_secs(1) {
            tasks::spawn_reload_logs(context.clone(), &logs_tx);
            log_refresh_pending = true;
            last_log_refresh = std::time::Instant::now();
        }
        while let Ok((session_id, sample)) = stats_rx.try_recv() {
            stats_poll_pending = false;
            needs_redraw = true;
            app.record_stats(session_id, sample);
        }
        let stats_settings = &context.app_config.runtime.stats;
        if stats_settings.enabled && app.data.runtime.pid_running {
            let poll_due = (!stats_poll_pending
                && last_stats_poll.elapsed() >= Duration::from_secs(1))
                || last_stats_poll.elapsed() >= Duration::from_secs(5);
            if poll_due {
                tasks::spawn_poll_stats(
                    stats_engine_for(&app, &context.app_config.runtime.engine),
                    stats_settings.host.clone(),
                    stats_settings.port,
                    app.data.runtime.session_id,
                    &stats_tx,
                );
                stats_poll_pending = true;
                last_stats_poll = std::time::Instant::now();
            }
        }
        while let Ok(tag) = version_rx.try_recv() {
            app.latest_version = Some(tag);
            needs_redraw = true;
        }
        while let Ok(engines) = engines_rx.try_recv() {
            app.engines = engines;
            needs_redraw = true;
        }
        needs_redraw |= app.tick();
        if app.take_needs_full_clear() {
            terminal.clear()?;
            needs_redraw = true;
        }
        if needs_redraw {
            terminal.draw(|frame| crate::tui::view::render(frame, &app))?;
            needs_redraw = false;
        }

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(100))? {
            needs_redraw = true;
            match event::read()? {
                Event::Resize(_, _) => {
                    app.needs_full_clear = true;
                }
                Event::Key(key) => {
                    let bulk_confirm_open = app.pending_bulk.is_some();
                    let action = crate::tui::keymap::action_for_key_with_import(
                        key,
                        app.active_view,
                        app.focused_panel,
                        &mut app.pending_chord,
                        bulk_confirm_open,
                        app.config_list.editing_search,
                        app.confirm.is_some(),
                        app.import_modal.is_some(),
                        app.rename_modal.is_some(),
                        app.qr_modal.is_some(),
                        app.settings_modal.as_ref().map(SettingsModalState::mode),
                    );
                    let bulk_to_run = if matches!(action, crate::tui::app::TuiAction::ConfirmBulk) {
                        app.pending_bulk
                    } else {
                        None
                    };
                    let confirmed_command = if matches!(action, crate::tui::app::TuiAction::Confirm)
                    {
                        app.pending_confirm_command()
                    } else {
                        None
                    };
                    let confirmed_source_delete =
                        if matches!(action, crate::tui::app::TuiAction::Confirm) {
                            if let Some(confirm) = &app.confirm {
                                if let crate::tui::app::ConfirmKind::DeleteSource(id) = confirm.kind
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
                    let confirmed_clear_events =
                        matches!(action, crate::tui::app::TuiAction::Confirm)
                            && app.confirm.as_ref().is_some_and(|confirm| {
                                matches!(confirm.kind, crate::tui::app::ConfirmKind::ClearEvents)
                            });
                    let confirmed_settings_restart =
                        matches!(action, crate::tui::app::TuiAction::Confirm)
                            && app.confirm.as_ref().is_some_and(|confirm| {
                                matches!(confirm.kind, ConfirmKind::RestartAfterSettings)
                            });
                    let direct_command = app.config_command_for_action(action);
                    let start_focused_id =
                        if matches!(action, crate::tui::app::TuiAction::StartFocused) {
                            app.focused_config()
                                .filter(|config| !config.is_deleted)
                                .map(|config| config.id)
                        } else {
                            None
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
                                .map(|s| (s.display_name().to_string(), s.value.clone()))
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
                    let open_import = matches!(action, crate::tui::app::TuiAction::OpenImportModal);
                    let open_settings =
                        matches!(action, crate::tui::app::TuiAction::OpenSettingsModal);
                    let save_settings_requested =
                        matches!(action, crate::tui::app::TuiAction::SettingsSave);
                    let open_rename = matches!(action, crate::tui::app::TuiAction::OpenRenameModal);
                    let rename_prefill = if open_rename {
                        app.focused_source().map(|source| {
                            (
                                source.id,
                                source.display_ref().to_string(),
                                source.display_name().to_string(),
                                source.name.clone().unwrap_or_default(),
                            )
                        })
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
                    let save_settings = save_settings_requested && app.prepare_settings_save();
                    if !save_settings_requested {
                        app.apply(action);
                    }
                    if open_import {
                        app.import_modal = Some(crate::tui::app::ImportModalState::default());
                        app.needs_full_clear = true;
                    }
                    if open_settings {
                        match ConfigEditSession::open(&context.runtime_paths.config_path) {
                            Ok(session) => {
                                app.settings_modal = Some(SettingsModalState::new(session));
                                app.needs_full_clear = true;
                            }
                            Err(error) => app.set_chrome_message(error, true),
                        }
                    }
                    if save_settings {
                        let settings_dirty = app
                            .settings_modal
                            .as_ref()
                            .is_some_and(|modal| modal.session.is_dirty());
                        if !settings_dirty {
                            if let Some(modal) = &mut app.settings_modal {
                                modal.notice = Some("No changes to save.".to_string());
                                modal.error = None;
                            }
                            continue;
                        }
                        let result = app
                            .settings_modal
                            .as_mut()
                            .map(|modal| modal.session.save());
                        match result {
                            Some(Ok(outcome)) => {
                                let runtime_restart =
                                    outcome.effects.contains(&SettingEffect::RuntimeRestart);
                                let daemon_restart =
                                    outcome.effects.contains(&SettingEffect::DaemonRestart);
                                let changed_count = outcome.changed_paths.len();
                                let changed_categories: BTreeSet<&str> = outcome
                                    .changed_paths
                                    .iter()
                                    .filter_map(|path| path.split('.').next())
                                    .collect();
                                context.app_config = outcome.config;
                                geo_lookup = crate::tui::data::build_geo_lookup(
                                    &context.app_config,
                                    &context.runtime_paths,
                                );
                                let reload_error =
                                    match TuiData::load(&context, app.config_list.include_deleted)
                                        .await
                                    {
                                        Ok(data) => {
                                            app.reload_data(data);
                                            None
                                        }
                                        Err(error) => Some(error),
                                    };
                                let mut message = if daemon_restart {
                                    format!(
                                        "saved {changed_count} setting(s); restart daemon to apply all changes"
                                    )
                                } else {
                                    format!("saved {changed_count} setting(s)")
                                };
                                let is_error = if let Some(error) = reload_error {
                                    message.push_str(&format!("; TUI reload failed: {error}"));
                                    true
                                } else {
                                    false
                                };
                                if let Some(modal) = &mut app.settings_modal {
                                    if is_error {
                                        modal.error = Some(message.clone());
                                        modal.notice = None;
                                    } else {
                                        modal.notice = Some(message.clone());
                                        modal.error = None;
                                    }
                                }
                                crate::app::events::record(
                                    &context.db,
                                    crate::app::events::LEVEL_INFO,
                                    crate::app::events::SOURCE_SETTINGS,
                                    "config_saved",
                                    message,
                                    None,
                                    app.data.runtime.session_id,
                                    Some(format!(
                                        "changed_fields={changed_count}; categories={}",
                                        changed_categories
                                            .into_iter()
                                            .collect::<Vec<_>>()
                                            .join(",")
                                    )),
                                )
                                .await;
                                if runtime_restart && app.data.runtime.pid_running {
                                    app.confirm = Some(ConfirmState {
                                        kind: ConfirmKind::RestartAfterSettings,
                                        prompt: "restart active runtime with new settings?"
                                            .to_string(),
                                    });
                                }
                            }
                            Some(Err(error)) => {
                                if let Some(modal) = &mut app.settings_modal {
                                    modal.error = Some(error);
                                }
                            }
                            None => {}
                        }
                    }
                    if matches!(action, crate::tui::app::TuiAction::ImportSubmit)
                        && let Some(import) = prepare_import_submission(&mut app)
                    {
                        app.import_modal = None;
                        tasks::spawn_import(
                            context.clone(),
                            import,
                            app.config_list.include_deleted,
                            &task_tx,
                        );
                    }
                    if matches!(action, crate::tui::app::TuiAction::ToggleDeletedFilter) {
                        tasks::spawn_reload_data(
                            context.clone(),
                            app.config_list.include_deleted,
                            &task_tx,
                        );
                    }
                    if matches!(action, crate::tui::app::TuiAction::StartTest(_)) {
                        tasks::spawn_test_batch(context.clone(), &mut app, &task_tx);
                    }
                    if let Some(op) = bulk_to_run {
                        tasks::run_bulk_op(&context, &mut app, op, &task_tx).await;
                    }
                    if matches!(action, crate::tui::app::TuiAction::RuntimeStop) {
                        tasks::spawn_runtime_stop(context.clone(), &mut app, &task_tx);
                    }
                    if matches!(action, crate::tui::app::TuiAction::RuntimeRestart) {
                        tasks::spawn_runtime_restart(context.clone(), &mut app, &task_tx);
                    }
                    if let Some((source_id, _source_value)) = focused_source_value {
                        tasks::spawn_source_refresh(
                            context.clone(),
                            source_id,
                            app.config_list.include_deleted,
                            &task_tx,
                        );
                    }
                    if !all_source_values.is_empty() {
                        let source_ids = all_source_values.into_iter().map(|(id, _)| id).collect();
                        tasks::spawn_source_refresh_all(
                            context.clone(),
                            source_ids,
                            app.config_list.include_deleted,
                            &task_tx,
                        );
                    }
                    if let Some((id, source_ref, current_name, input)) = rename_prefill {
                        app.rename_modal = Some(crate::tui::app::RenameModalState {
                            source_id: id,
                            source_ref,
                            current_name,
                            input,
                            error: None,
                        });
                        app.needs_full_clear = true;
                    }
                    if let Some((source_id, name)) = rename_submit {
                        app.rename_modal = None;
                        tasks::run_source_rename(&context, &mut app, source_id, name, &task_tx)
                            .await;
                    }
                    if let Some(source_id) = confirmed_source_delete {
                        tasks::run_source_delete(&context, &mut app, source_id, &task_tx).await;
                    }
                    if confirmed_clear_events {
                        tasks::run_clear_events(&context, &mut app, &logs_tx).await;
                    }
                    if let Some((config_id, config_name)) = qr_config_id {
                        tasks::open_qr_for_config(&context, &mut app, config_id, config_name).await;
                    }
                    if let Some((source_name, source_url)) = qr_source {
                        tasks::open_qr_for_source(&mut app, source_name, source_url);
                    }
                    if let Some(config_id) = copy_focused_id {
                        tasks::copy_config_uri(&context, &mut app, config_id).await;
                    }
                    if let Some((source_name, source_url)) = copy_focused_source {
                        tasks::copy_source_uri(&mut app, source_name, source_url);
                    }
                    if matches!(action, crate::tui::app::TuiAction::OpenQrApiUrl) {
                        tasks::open_qr_for_api_url(&mut app);
                    }
                    if matches!(action, crate::tui::app::TuiAction::CopyApiUrl) {
                        tasks::copy_api_url(&mut app);
                    }
                    if let Some(command) = confirmed_command.or(direct_command) {
                        tasks::run_config_command(&context, &mut app, command, &task_tx).await;
                    }
                    if confirmed_settings_restart {
                        tasks::spawn_runtime_restart(context.clone(), &mut app, &task_tx);
                    }
                    if let Some(config_id) = start_focused_id {
                        tasks::spawn_runtime_start_config(
                            context.clone(),
                            &mut app,
                            &task_tx,
                            config_id,
                        );
                    }
                }
                Event::Paste(text) => {
                    if app.settings_modal.is_some() {
                        app.append_settings_text(&text);
                    } else {
                        app.append_import_text(&text);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn prepare_import_submission(app: &mut TuiApp) -> Option<tasks::TuiImport> {
    use crate::tui::app::ImportModalStep;

    let modal = app.import_modal.as_ref()?;
    let step = modal.step.clone();
    let input = modal.input.trim().to_string();

    match step {
        ImportModalStep::Link => {
            if input.is_empty() {
                set_import_error(app, "paste a config link or subscription URL");
                return None;
            }

            if crate::support::url::looks_like_url(&input) {
                let valid_url = url::Url::parse(&input)
                    .ok()
                    .is_some_and(|url| url.host_str().is_some());
                if !valid_url {
                    set_import_error(app, "invalid HTTP(S) subscription URL");
                    return None;
                }
                let random_ref = crate::support::refs::generate_ref();
                let suggested_name = format!("sub-{}", &random_ref[..6]);
                app.import_modal = Some(crate::tui::app::ImportModalState {
                    step: ImportModalStep::SubscriptionName {
                        url: input,
                        suggested_name,
                    },
                    input: String::new(),
                    error: None,
                });
                app.needs_full_clear = true;
                return None;
            }

            if input.contains('\n') || input.split_once("://").is_none() {
                set_import_error(app, "only one supported config link can be added");
                return None;
            }
            match crate::app::import::load_single_node(&input) {
                Ok((source, node)) => Some(tasks::TuiImport::Config {
                    source,
                    node: Box::new(node),
                }),
                Err(_) => {
                    set_import_error(app, "unsupported or invalid config link");
                    None
                }
            }
        }
        ImportModalStep::SubscriptionName {
            url,
            suggested_name,
        } => {
            let name = if input.is_empty() {
                suggested_name
            } else {
                input
            };
            Some(tasks::TuiImport::Subscription { url, name })
        }
    }
}

fn set_import_error(app: &mut TuiApp, error: &str) {
    if let Some(modal) = &mut app.import_modal {
        modal.error = Some(error.to_string());
    }
}

/// Resolve which stats backend to sample. Hy2 always runs on managed sing-box;
/// otherwise the configured runtime engine decides. Mirrors the engine
/// selection in `resolve_runtime_engine`.
fn stats_engine_for(app: &TuiApp, configured_engine: &str) -> tasks::StatsEngine {
    let active_protocol = app
        .data
        .runtime
        .active_config_id
        .and_then(|id| app.data.configs.iter().find(|config| config.id == id))
        .map(|config| config.protocol.to_ascii_lowercase());
    if active_protocol.as_deref() == Some("hy2") {
        return tasks::StatsEngine::Singbox;
    }
    match configured_engine {
        "sing-box" => tasks::StatsEngine::Singbox,
        _ => tasks::StatsEngine::Xray,
    }
}

#[derive(Default)]
struct TaskDrain {
    /// Any task event was applied, so the frame should be redrawn.
    any: bool,
    /// A full data reload landed, so location enrichment should re-run.
    reloaded: bool,
}

/// Drain queued task events into the app.
fn drain_task_events(
    app: &mut TuiApp,
    task_rx: &mut mpsc::UnboundedReceiver<TuiTaskEvent>,
) -> TaskDrain {
    let mut drain = TaskDrain::default();
    while let Ok(event) = task_rx.try_recv() {
        drain.any = true;
        drain.reloaded |= matches!(
            event,
            TuiTaskEvent::Completed { data: Some(_), .. }
                | TuiTaskEvent::Failed { data: Some(_), .. }
        );
        app.apply_task_event(event);
    }
    drain
}
