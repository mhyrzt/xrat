use crate::app::AppError;
use crate::app::commands::output::{self, Align, Cell, Column};
use crate::app::config::TunSplitMode;
use crate::app::context::AppContext;
use crate::app::services::split_tunnel;
use crate::app::tun_privileges::{self, TunPrivilegeFile};
use crate::cli::{
    ListFormat, TunAction, TunArgs, TunKillswitchActionArg, TunKillswitchArgs, TunModeArgs,
    TunSetSplitModeArgs, TunSetupArgs, TunSplitAction, TunSplitAppsArgs, TunSplitArgs,
    TunSplitClearArgs, TunSplitClearTarget, TunSplitListArgs, TunSplitListTarget,
    TunSplitListViewTarget, TunSplitModifyArgs, TunStatusArgs,
};
#[cfg(not(any(target_os = "macos", windows)))]
use xrat_support::process::Stdio;

pub async fn run(context: &AppContext, args: &TunArgs) -> crate::app::Result<()> {
    match &args.action {
        TunAction::Status(status) => status_command(context, status).await,
        TunAction::Setup(setup) => setup_command(context, setup).await,
        TunAction::Enable(mode) => enabled_command(context, true, mode).await,
        TunAction::Disable(mode) => enabled_command(context, false, mode).await,
        TunAction::Mode(mode) => set_split_mode_command(context, mode).await,
        TunAction::Split(split) => split_command(context, split).await,
        TunAction::Killswitch(killswitch) => killswitch_command(context, killswitch).await,
    }
}

async fn enabled_command(
    context: &AppContext,
    enabled: bool,
    args: &TunModeArgs,
) -> crate::app::Result<()> {
    let mut context = context.clone();
    let outcome = if args.kill_switch || (enabled && args.mode.is_some()) {
        let target_mode: Option<TunSplitMode> = args.mode.map(Into::into);
        let kill_switch = args.kill_switch;
        crate::app::services::tun_control::mutate_tun_settings(&mut context, false, |tun| {
            tun.enabled = enabled;
            if let Some(mode) = target_mode {
                tun.split_mode = mode;
            }
            if kill_switch && enabled {
                tun.kill_switch = crate::app::config::TunKillSwitchMode::Auto;
            }
            Ok(())
        })
        .await?
    } else {
        crate::app::services::tun_control::apply(&mut context, Some(enabled), false).await?
    };
    print_tun_outcome(&outcome, args.json)
}

async fn set_split_mode_command(
    context: &AppContext,
    args: &TunSetSplitModeArgs,
) -> crate::app::Result<()> {
    let mut context = context.clone();
    let target_mode: TunSplitMode = args.mode.into();
    let outcome =
        crate::app::services::tun_control::mutate_tun_settings(&mut context, false, |tun| {
            tun.split_mode = target_mode;
            Ok(())
        })
        .await?;
    print_tun_outcome(&outcome, args.json)
}

async fn killswitch_command(
    context: &AppContext,
    args: &TunKillswitchArgs,
) -> crate::app::Result<()> {
    match args.state {
        TunKillswitchActionArg::Status => {
            let ports = xrat_support::readiness::RuntimeProcessPorts::default();
            let is_active = ports.firewall.is_active().unwrap_or(false);
            let backend = ports.firewall.backend_name();
            let configured = context.app_config.runtime.tun.kill_switch;
            let allow_lan = context.app_config.runtime.tun.allow_lan;

            if args.json {
                let value = serde_json::json!({
                    "active": is_active,
                    "configured": configured.as_str(),
                    "backend": backend,
                    "allow_lan": allow_lan,
                });
                println!("{}", serde_json::to_string_pretty(&value)?);
                return Ok(());
            }

            let color = output::color_enabled();
            let rows: Vec<(&str, String)> = vec![
                (
                    "kill switch state",
                    if is_active {
                        "active (blocking non-TUN traffic)"
                    } else {
                        "inactive"
                    }
                    .to_string(),
                ),
                ("configured mode", configured.as_str().to_string()),
                ("firewall backend", backend.to_string()),
                (
                    "LAN access",
                    if allow_lan { "allowed" } else { "blocked" }.to_string(),
                ),
            ];
            println!(
                "{}",
                output::format_kv(Some("TUN Kill Switch Status"), &rows, color)
            );
            Ok(())
        }
        TunKillswitchActionArg::On => {
            let mut context = context.clone();
            let outcome = crate::app::services::tun_control::mutate_tun_settings(
                &mut context,
                false,
                |tun| {
                    tun.kill_switch = crate::app::config::TunKillSwitchMode::Auto;
                    Ok(())
                },
            )
            .await?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&outcome)?);
            } else {
                println!(
                    "{}",
                    output::success(
                        "TUN kill switch enabled (fail-closed mode active).",
                        output::color_enabled()
                    )
                );
            }
            Ok(())
        }
        TunKillswitchActionArg::Off => {
            let mut context = context.clone();
            let outcome = crate::app::services::tun_control::mutate_tun_settings(
                &mut context,
                false,
                |tun| {
                    tun.kill_switch = crate::app::config::TunKillSwitchMode::Off;
                    Ok(())
                },
            )
            .await?;
            let ports = xrat_support::readiness::RuntimeProcessPorts::default();
            ports.firewall.remove_killswitch().map_err(|error| {
                AppError::InvalidArgument(format!("Failed to deactivate TUN kill switch: {error}"))
            })?;
            if args.json {
                println!("{}", serde_json::to_string_pretty(&outcome)?);
            } else {
                println!(
                    "{}",
                    output::success("TUN kill switch disabled.", output::color_enabled())
                );
            }
            Ok(())
        }
        TunKillswitchActionArg::Reset => {
            let ports = xrat_support::readiness::RuntimeProcessPorts::default();
            ports.firewall.remove_killswitch().map_err(|e| {
                AppError::InvalidArgument(format!("Failed to reset kill switch rules: {e}"))
            })?;
            if let Some(mut record) = crate::app::runtime_service::tun_ownership::load_ownership(
                &context.runtime_paths.runtime_dir,
            ) {
                record.kill_switch_active = false;
                let _ = crate::app::runtime_service::tun_ownership::save_ownership(
                    &context.runtime_paths.runtime_dir,
                    &record,
                );
            }
            if args.json {
                println!("{}", serde_json::json!({ "reset": true, "active": false }));
            } else {
                println!(
                    "{}",
                    output::success(
                        "Kill switch rules cleared. Network access restored.",
                        output::color_enabled()
                    )
                );
            }
            Ok(())
        }
    }
}

fn print_tun_outcome(
    outcome: &crate::app::daemon::ipc::TunStatePayload,
    json: bool,
) -> crate::app::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(outcome)?);
    } else {
        println!(
            "{}",
            output::success(
                crate::app::services::tun_control::message(outcome),
                output::color_enabled()
            )
        );
    }
    Ok(())
}

async fn split_command(context: &AppContext, args: &TunSplitArgs) -> crate::app::Result<()> {
    match &args.action {
        TunSplitAction::Status(status) => split_status_command(context, status).await,
        TunSplitAction::Mode(mode) => set_split_mode_command(context, mode).await,
        TunSplitAction::List(list) => split_list_command(context, list),
        TunSplitAction::Add(modify) => split_add_command(context, modify).await,
        TunSplitAction::Remove(modify) => split_remove_command(context, modify).await,
        TunSplitAction::Clear(clear) => split_clear_command(context, clear).await,
        TunSplitAction::Apps(apps) => split_apps_command(apps),
    }
}

async fn split_status_command(
    context: &AppContext,
    args: &TunStatusArgs,
) -> crate::app::Result<()> {
    let tun = &context.app_config.runtime.tun;
    let compiled = split_tunnel::compile_split_rules(tun);
    let state = current_state(context).await?;
    let color = output::color_enabled();

    if args.json {
        let payload = serde_json::json!({
            "tun_enabled": tun.enabled,
            "tun_active": state.active,
            "split_mode": tun.split_mode.as_str(),
            "blacklist": tun.blacklist,
            "whitelist": tun.whitelist,
            "resolved": {
                "xray_processes": compiled.xray_processes,
                "singbox_process_name": compiled.singbox_process_names,
                "singbox_process_path": compiled.singbox_process_paths,
                "singbox_process_path_regex": compiled.singbox_process_path_regexes,
            },
            "warnings": compiled.warnings,
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    let rows: Vec<(&str, String)> = vec![
        ("split mode", tun.split_mode.as_str().to_string()),
        (
            "TUN state",
            if state.active {
                "active".to_string()
            } else if tun.enabled {
                "enabled (disconnected)".to_string()
            } else {
                "disabled".to_string()
            },
        ),
        (
            "blacklist",
            if tun.blacklist.is_empty() {
                "none".to_string()
            } else {
                format!("{} ({})", tun.blacklist.len(), tun.blacklist.join(", "))
            },
        ),
        (
            "whitelist",
            if tun.whitelist.is_empty() {
                "none".to_string()
            } else {
                format!("{} ({})", tun.whitelist.len(), tun.whitelist.join(", "))
            },
        ),
        (
            "active matchers",
            if compiled.xray_processes.is_empty() {
                "none".to_string()
            } else {
                compiled.xray_processes.join(", ")
            },
        ),
    ];
    println!(
        "{}",
        output::format_kv(Some("TUN split tunneling"), &rows, color)
    );
    for warning in &compiled.warnings {
        println!("{}", output::warn(warning, color));
    }
    Ok(())
}

fn split_list_command(context: &AppContext, args: &TunSplitListArgs) -> crate::app::Result<()> {
    let tun = &context.app_config.runtime.tun;
    let desktop_dirs = split_tunnel::desktop_search_dirs();
    let mut entries = Vec::new();

    if matches!(
        args.list,
        TunSplitListViewTarget::All | TunSplitListViewTarget::Blacklist
    ) {
        for item in &tun.blacklist {
            let resolved = split_tunnel::normalize_app_specifier_with_dirs(item, &desktop_dirs)
                .unwrap_or_else(|_| vec![item.clone()]);
            entries.push((
                "blacklist",
                item.clone(),
                resolved.join(", "),
                tun.split_mode == TunSplitMode::Blacklist,
            ));
        }
    }
    if matches!(
        args.list,
        TunSplitListViewTarget::All | TunSplitListViewTarget::Whitelist
    ) {
        for item in &tun.whitelist {
            let resolved = split_tunnel::normalize_app_specifier_with_dirs(item, &desktop_dirs)
                .unwrap_or_else(|_| vec![item.clone()]);
            entries.push((
                "whitelist",
                item.clone(),
                resolved.join(", "),
                tun.split_mode == TunSplitMode::Whitelist,
            ));
        }
    }

    if args.json || matches!(args.format, ListFormat::Json) {
        let json_entries: Vec<serde_json::Value> = entries
            .iter()
            .map(|(list, spec, resolved, active)| {
                serde_json::json!({
                    "list": list,
                    "specifier": spec,
                    "resolved": resolved,
                    "active": active,
                })
            })
            .collect();
        let value = serde_json::json!({
            "split_mode": tun.split_mode.as_str(),
            "entries": json_entries,
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    if entries.is_empty() {
        println!(
            "{}",
            output::empty_message("No split tunneling applications configured.")
        );
        return Ok(());
    }

    match args.format {
        ListFormat::Tsv => {
            println!("list\tspecifier\tresolved\tactive");
            for (list, spec, resolved, active) in entries {
                println!("{list}\t{spec}\t{resolved}\t{}", output::bool_label(active));
            }
        }
        ListFormat::Table => {
            let columns = [
                Column {
                    header: "LIST",
                    align: Align::Left,
                },
                Column {
                    header: "SPECIFIER",
                    align: Align::Left,
                },
                Column {
                    header: "RESOLVED MATCHERS",
                    align: Align::Left,
                },
                Column {
                    header: "ACTIVE",
                    align: Align::Left,
                },
            ];
            let rows: Vec<Vec<Cell>> = entries
                .into_iter()
                .map(|(list, spec, resolved, active)| {
                    vec![
                        Cell::plain(list),
                        Cell::plain(spec),
                        Cell::plain(resolved),
                        Cell::plain(output::bool_label(active)),
                    ]
                })
                .collect();
            println!(
                "{}",
                output::format_table(&columns, &rows, output::color_enabled())
            );
        }
        ListFormat::Json => unreachable!(),
    }
    Ok(())
}

async fn split_add_command(
    context: &AppContext,
    args: &TunSplitModifyArgs,
) -> crate::app::Result<()> {
    let mut normalized_inputs = Vec::new();
    for raw in &args.apps {
        let trimmed = raw.trim();
        let _ =
            split_tunnel::normalize_app_specifier(trimmed).map_err(AppError::InvalidArgument)?;
        if !normalized_inputs
            .iter()
            .any(|existing: &String| existing == trimmed)
        {
            normalized_inputs.push(trimmed.to_string());
        }
    }

    let mut context = context.clone();
    let outcome =
        crate::app::services::tun_control::mutate_tun_settings(&mut context, false, |tun| {
            let target = match args.list {
                TunSplitListTarget::Blacklist => &mut tun.blacklist,
                TunSplitListTarget::Whitelist => &mut tun.whitelist,
            };
            for item in &normalized_inputs {
                if !target.contains(item) {
                    target.push(item.clone());
                }
            }
            Ok(())
        })
        .await?;
    print_tun_outcome(&outcome, args.json)
}

async fn split_remove_command(
    context: &AppContext,
    args: &TunSplitModifyArgs,
) -> crate::app::Result<()> {
    let to_remove: Vec<String> = args
        .apps
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if to_remove.is_empty() {
        return Err(AppError::InvalidArgument(
            "at least one non-empty application specifier is required".to_string(),
        ));
    }

    let mut context = context.clone();
    let outcome =
        crate::app::services::tun_control::mutate_tun_settings(&mut context, false, |tun| {
            let target = match args.list {
                TunSplitListTarget::Blacklist => &mut tun.blacklist,
                TunSplitListTarget::Whitelist => &mut tun.whitelist,
            };
            target.retain(|entry| !to_remove.iter().any(|rem| rem == entry.trim()));
            Ok(())
        })
        .await?;
    print_tun_outcome(&outcome, args.json)
}

async fn split_clear_command(
    context: &AppContext,
    args: &TunSplitClearArgs,
) -> crate::app::Result<()> {
    let mut context = context.clone();
    let outcome =
        crate::app::services::tun_control::mutate_tun_settings(&mut context, false, |tun| {
            match args.list {
                TunSplitClearTarget::Blacklist => tun.blacklist.clear(),
                TunSplitClearTarget::Whitelist => tun.whitelist.clear(),
                TunSplitClearTarget::Both => {
                    tun.blacklist.clear();
                    tun.whitelist.clear();
                }
            }
            Ok(())
        })
        .await?;
    print_tun_outcome(&outcome, args.json)
}

fn split_apps_command(args: &TunSplitAppsArgs) -> crate::app::Result<()> {
    let (include_running, include_desktop) = match (args.running, args.desktop) {
        (false, false) => (true, true),
        other => other,
    };
    let apps =
        split_tunnel::discover_apps(include_running, include_desktop, args.search.as_deref());

    if args.json || matches!(args.format, ListFormat::Json) {
        println!("{}", serde_json::to_string_pretty(&apps)?);
        return Ok(());
    }

    if apps.is_empty() {
        println!(
            "{}",
            output::empty_message("No matching applications found.")
        );
        return Ok(());
    }

    match args.format {
        ListFormat::Tsv => {
            println!("specifier\tname\tsource\texe_path\tdesktop_id");
            for app in apps {
                println!(
                    "{}\t{}\t{}\t{}\t{}",
                    app.rule_entry,
                    app.name,
                    app.source.as_str(),
                    output::dash(app.exec_path.as_deref()),
                    output::dash(app.desktop_id.as_deref()),
                );
            }
        }
        ListFormat::Table => {
            let columns = [
                Column {
                    header: "SPECIFIER",
                    align: Align::Left,
                },
                Column {
                    header: "NAME",
                    align: Align::Left,
                },
                Column {
                    header: "SOURCE",
                    align: Align::Left,
                },
                Column {
                    header: "EXECUTABLE / DESKTOP",
                    align: Align::Left,
                },
            ];
            let rows: Vec<Vec<Cell>> = apps
                .into_iter()
                .map(|app| {
                    let detail = app
                        .exec_path
                        .or(app.desktop_id)
                        .unwrap_or_else(|| "-".to_string());
                    vec![
                        Cell::plain(app.rule_entry),
                        Cell::plain(output::truncate(&app.name, 36)),
                        Cell::plain(app.source.as_str()),
                        Cell::plain(detail),
                    ]
                })
                .collect();
            println!(
                "{}",
                output::format_table(&columns, &rows, output::color_enabled())
            );
        }
        ListFormat::Json => unreachable!(),
    }
    Ok(())
}

async fn status_command(context: &AppContext, args: &TunStatusArgs) -> crate::app::Result<()> {
    let runtime = &context.app_config.runtime;
    let ports = xrat_support::readiness::RuntimeProcessPorts::default();
    let engine_error = crate::app::services::tun::check_engine(context, &ports)
        .err()
        .map(|error| error.to_string());
    let engine_version = if runtime.engine == "xray" {
        crate::app::services::runtime_tuning::xray_binary_version_with_spawner(
            &context.runtime_paths.xray_path,
            ports.spawner.clone(),
        )
        .map(|(major, minor, patch)| format!("{major}.{minor}.{patch}"))
    } else {
        None
    };
    let state = current_state(context).await?;
    let files = tun_privileges::required_files(context);
    let color = output::color_enabled();
    let supported = tun_privileges::capabilities_supported();
    let macos_ready = cfg!(target_os = "macos")
        && tun_privileges::inspect_process_privileges(std::process::id())
            .is_some_and(|(_, ready)| ready);

    let service_state = tun_privileges::systemd_service_status();
    let service_ready = service_state.as_ref().map(|(_, r)| *r).unwrap_or(true);
    let daemon_state = daemon_process_status(&context.runtime_paths.runtime_dir).await;
    let daemon_ready = daemon_state.as_ref().is_none_or(|(_, _, ready)| *ready);

    if args.json {
        let entries: Vec<serde_json::Value> = files
            .iter()
            .map(|file| {
                let capabilities = tun_privileges::file_capabilities(&file.path);
                serde_json::json!({
                    "role": file.label,
                    "path": file.path.display().to_string(),
                    "capabilities": capabilities,
                    "required_capabilities": supported.then_some(file.capabilities),
                    "ready": capabilities
                        .as_deref()
                        .map(tun_privileges::has_net_admin)
                        .unwrap_or(macos_ready),
                    "process_routing_ready": capabilities
                        .as_deref()
                        .map(tun_privileges::has_process_routing_caps)
                        .unwrap_or(false),
                })
            })
            .collect();
        let all_files_ready = entries
            .iter()
            .all(|e| e["ready"].as_bool().unwrap_or(false));
        let value = serde_json::json!({
            "engine": runtime.engine.clone(),
            "tun_enabled": runtime.tun.enabled,
            "tun_active": state.active,
            "active_interface": state.interface,
            "active_config_ref": state.active_config_ref,
            "split_mode": runtime.tun.split_mode.as_str(),
            "blacklist": runtime.tun.blacklist,
            "whitelist": runtime.tun.whitelist,
            "kill_switch": runtime.tun.kill_switch.as_str(),
            "kill_switch_active": state.kill_switch_active,
            "allow_lan": runtime.tun.allow_lan,
            "engine_version": engine_version,
            "engine_error": engine_error,
            "interface": runtime.tun.interface_name.clone(),
            "capabilities_supported": supported,
            "service_ready": service_ready,
            "daemon": daemon_state.map(|(pid, status, ready)| serde_json::json!({
                "pid": pid,
                "status": status,
                "ready": ready,
            })),
            "ready": (supported || macos_ready) && engine_error.is_none() && all_files_ready && service_ready && daemon_ready,
            "files": entries,
        });
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    let mut rows: Vec<(&str, String)> = vec![
        (
            "engine",
            format!(
                "{}{}",
                runtime.engine,
                engine_version
                    .as_ref()
                    .map(|version| format!(" {version}"))
                    .unwrap_or_default()
            ),
        ),
        (
            "active capture",
            if state.active { "yes" } else { "no" }.into(),
        ),
        (
            "active config",
            state
                .active_config_ref
                .clone()
                .unwrap_or_else(|| "disconnected".into()),
        ),
        (
            "active interface",
            state.interface.clone().unwrap_or_else(|| "none".into()),
        ),
        (
            "engine check",
            if engine_error.is_some() {
                "blocked"
            } else {
                "ready"
            }
            .into(),
        ),
        (
            "configured TUN",
            if runtime.tun.enabled { "yes" } else { "no" }.to_string(),
        ),
        ("configured interface", runtime.tun.interface_name.clone()),
        ("split mode", runtime.tun.split_mode.as_str().to_string()),
        (
            "blacklist",
            if runtime.tun.blacklist.is_empty() {
                "none".to_string()
            } else {
                format!(
                    "{} ({})",
                    runtime.tun.blacklist.len(),
                    runtime.tun.blacklist.join(", ")
                )
            },
        ),
        (
            "whitelist",
            if runtime.tun.whitelist.is_empty() {
                "none".to_string()
            } else {
                format!(
                    "{} ({})",
                    runtime.tun.whitelist.len(),
                    runtime.tun.whitelist.join(", ")
                )
            },
        ),
        (
            "kill switch",
            if state.kill_switch_active {
                format!("active ({}) [fail-closed]", ports.firewall.backend_name())
            } else if runtime.tun.kill_switch.is_enabled() {
                format!(
                    "configured ({}) [standby]",
                    runtime.tun.kill_switch.as_str()
                )
            } else {
                "disabled".to_string()
            },
        ),
        (
            "LAN access",
            if runtime.tun.allow_lan {
                "allowed"
            } else {
                "blocked"
            }
            .to_string(),
        ),
    ];
    if let Some((desc, _ready)) = &service_state {
        rows.push(("systemd service", desc.to_string()));
    }
    if let Some((pid, status, _)) = &daemon_state {
        rows.push(("daemon process", format!("{pid}: {status}")));
    }
    let mut missing = !service_ready || !daemon_ready;
    for file in &files {
        let (state, ready) = capability_state(file);
        missing |= !ready;
        rows.push((file.label, format!("{}  {state}", file.path.display())));
    }
    println!("{}", output::format_kv(Some("TUN status"), &rows, color));

    if let Some(error) = engine_error {
        println!("{}", output::notice(&error, color));
    }
    if runtime.tun.enabled && !state.active {
        println!(
            "{}",
            output::notice(
                if state.active_config_ref.is_some() {
                    "TUN is configured but inactive. Run `xrat tun enable` to apply it to the current connection."
                } else {
                    "Disconnected: TUN will apply on the next connect."
                },
                color
            )
        );
    }
    if !supported {
        println!(
            "{}",
            output::notice(
                if cfg!(target_os = "macos") {
                    "macOS managed TUN requires sing-box and a root runtime owner; use explicit config and database paths with sudo"
                } else {
                    "managed TUN is unavailable on this platform"
                },
                color,
            )
        );
    } else if missing {
        println!(
            "{}",
            output::notice("run `xrat tun setup` to grant CAP_NET_ADMIN", color)
        );
    }
    Ok(())
}

async fn current_state(
    context: &AppContext,
) -> crate::app::Result<crate::app::daemon::ipc::TunStatePayload> {
    #[cfg(unix)]
    {
        let socket =
            crate::app::daemon::ipc::default_socket_path(&context.runtime_paths.runtime_dir);
        if crate::app::services::runtime_control::tui_uses_daemon(&socket).await? {
            let response = crate::app::daemon::ipc::runtime_status_daemon(&socket).await?;
            if !response.ok {
                return Err(AppError::InvalidArgument(response.message));
            }
            return response.payload.and_then(|payload| payload.tun).ok_or_else(|| AppError::InvalidArgument(
                "The running daemon cannot report active TUN state. Restart it once after upgrading, then retry.".into(),
            ));
        }
    }
    let snapshot = crate::app::runtime_service::RuntimeService::new(context)
        .status()
        .await?;
    Ok(crate::app::services::tun::capture_state(
        context,
        &snapshot,
        &xrat_support::readiness::RuntimeProcessPorts::default(),
    ))
}

async fn daemon_process_status(runtime_dir: &std::path::Path) -> Option<(u32, &'static str, bool)> {
    #[cfg(target_os = "linux")]
    {
        let socket_path = crate::app::daemon::ipc::default_socket_path(runtime_dir);
        let stream = tokio::time::timeout(
            std::time::Duration::from_secs(1),
            tokio::net::UnixStream::connect(socket_path),
        )
        .await
        .ok()?
        .ok()?;
        let pid = u32::try_from(stream.peer_cred().ok()?.pid()?).ok()?;
        let (status, ready) = tun_privileges::inspect_process_privileges(pid).unwrap_or((
            "unknown (could not inspect running daemon privileges)",
            false,
        ));
        Some((pid, status, ready))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = runtime_dir;
        None
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    #[tokio::test]
    async fn tun_status_inspects_daemon_peer_privileges() {
        let runtime_dir = tempfile::tempdir().unwrap();
        assert!(
            super::daemon_process_status(runtime_dir.path())
                .await
                .is_none()
        );
        let socket_path = crate::app::daemon::ipc::default_socket_path(runtime_dir.path());
        let _listener = tokio::net::UnixListener::bind(socket_path).unwrap();
        let (pid, status, ready) = super::daemon_process_status(runtime_dir.path())
            .await
            .unwrap();
        assert_eq!(pid, std::process::id());
        assert_eq!(
            Some((status, ready)),
            crate::app::tun_privileges::inspect_process_privileges(pid)
        );
    }
}

fn capability_state(file: &TunPrivilegeFile) -> (String, bool) {
    #[cfg(target_os = "macos")]
    {
        return (
            "macOS uses a root runtime owner, not file capabilities".into(),
            file.path.is_file() && tun_privileges::macos_root(),
        );
    }
    #[cfg(not(target_os = "macos"))]
    {
        if !file.path.is_file() {
            return (
                "not found; install the core or set [paths] to an absolute path".to_string(),
                false,
            );
        }
        match tun_privileges::file_capabilities(&file.path) {
            None => (
                "unknown (install libcap tools to inspect)".to_string(),
                false,
            ),
            Some(capabilities) if tun_privileges::has_net_admin(&capabilities) => {
                (format!("ready ({capabilities})"), true)
            }
            Some(capabilities) if capabilities.is_empty() => {
                ("missing (run `xrat tun setup`)".to_string(), false)
            }
            Some(capabilities) => (format!("missing CAP_NET_ADMIN ({capabilities})"), false),
        }
    }
}

async fn setup_command(context: &AppContext, args: &TunSetupArgs) -> crate::app::Result<()> {
    #[cfg(windows)]
    {
        let _ = (context, args);
        Err(AppError::InvalidArgument("Windows managed TUN setup uses the protected machine profile. Install the machine service, stop it, then run `xrat --profile machine tun setup` in an elevated administrator shell.".into()))
    }
    #[cfg(target_os = "macos")]
    {
        let _ = args;
        if context.app_config.runtime.engine != "sing-box" {
            return Err(AppError::InvalidArgument(
                "macOS managed TUN requires sing-box".into(),
            ));
        }
        println!(
            "macOS managed TUN requires a root runtime owner. Run xrat with sudo and explicit --config and --database paths. Set [runtime].engine = \"sing-box\" and [runtime.tun].interface_name = \"utun99\". File capabilities are not used on macOS."
        );
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", windows)))]
    setup_capabilities(context, args).await
}

#[cfg(not(any(target_os = "macos", windows)))]
async fn setup_capabilities(context: &AppContext, args: &TunSetupArgs) -> crate::app::Result<()> {
    if !tun_privileges::capabilities_supported() {
        return Err(AppError::InvalidArgument(
            "file capabilities are Linux-only; configure TUN privileges manually on this platform"
                .to_string(),
        ));
    }
    if context.app_config.runtime.engine == "xray" {
        crate::app::services::runtime_tuning::ensure_xray_tun_supported_with_spawner(
            &context.runtime_paths.xray_path,
            std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
        )?;
    } else if context.app_config.runtime.engine != "sing-box" {
        return Err(AppError::InvalidArgument(
            "Managed TUN requires Xray or sing-box. Select a supported engine in settings.".into(),
        ));
    }
    let files = tun_privileges::required_files(context);
    let color = output::color_enabled();

    let Some(engine_file) = files.first() else {
        return Err(AppError::InvalidArgument(
            "could not determine the engine binary path".to_string(),
        ));
    };
    if !engine_file.path.is_file() {
        return Err(AppError::InvalidArgument(format!(
            "engine binary not found at {}; install it (for example `xrat install xray`) or set [paths] to an absolute path",
            engine_file.path.display()
        )));
    }
    let files: Vec<TunPrivilegeFile> = files
        .into_iter()
        .filter(|file| file.path.is_file())
        .collect();

    let commands: Vec<String> = files
        .iter()
        .map(|file| format!("sudo setcap {} {}", file.capabilities, file.path.display()))
        .collect();

    if args.dry_run {
        for command in &commands {
            println!("{command}");
        }
        #[cfg(target_os = "linux")]
        {
            let user_dir = crate::app::commands::daemon_install::systemd_user_dir_with_env(
                &xrat_support::env::SystemEnvVars,
            )?;
            if user_dir.is_dir() {
                let override_path = user_dir
                    .join(crate::app::commands::daemon_install::DAEMON_OVERRIDE_DIR_NAME)
                    .join(crate::app::commands::daemon_install::DAEMON_TUN_OVERRIDE_FILE_NAME);
                println!("write {} (NoNewPrivileges=false)", override_path.display());
                println!("systemctl --user daemon-reload");
            }
        }
        return Ok(());
    }

    println!(
        "{}",
        output::notice("granting TUN capabilities with `sudo setcap`", color)
    );
    // `setcap` accepts a single file per capability specification, so each file
    // gets its own invocation (sudo caches the credential across them).
    for (file, printable) in files.iter().zip(&commands) {
        let mut command = xrat_support::process::Command::new("sudo");
        command.arg("setcap").arg(file.capabilities).arg(&file.path);
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        let status = command.status()?;
        if !status.success() {
            return Err(AppError::InvalidArgument(format!(
                "`{printable}` failed ({status}); run it manually and check your privileges"
            )));
        }
    }

    println!("{}", output::success("Granted TUN capabilities.", color));

    #[cfg(target_os = "linux")]
    {
        let user_dir = crate::app::commands::daemon_install::systemd_user_dir_with_env(
            &xrat_support::env::SystemEnvVars,
        )?;
        let service_file = user_dir.join(crate::app::commands::daemon_install::DAEMON_SERVICE_NAME);
        let override_dir =
            user_dir.join(crate::app::commands::daemon_install::DAEMON_OVERRIDE_DIR_NAME);
        let override_file =
            override_dir.join(crate::app::commands::daemon_install::DAEMON_TUN_OVERRIDE_FILE_NAME);

        if service_file.is_file() || user_dir.is_dir() {
            std::fs::create_dir_all(&override_dir)?;
            std::fs::write(
                &override_file,
                crate::app::commands::daemon_install::DAEMON_TUN_OVERRIDE_TEMPLATE,
            )?;
            let status = xrat_support::process::Command::new("systemctl")
                .args(["--user", "daemon-reload"])
                .status()?;
            if !status.success() {
                return Err(AppError::InvalidArgument(format!(
                    "systemctl --user daemon-reload failed ({status}); run it before restarting the daemon"
                )));
            }
        }
    }

    if !args.dry_run {
        let mut rows: Vec<(&str, String)> = Vec::new();
        for file in &files {
            let (state, _) = capability_state(file);
            rows.push((file.label, format!("{}  {state}", file.path.display())));
        }
        if let Some((desc, _)) = tun_privileges::systemd_service_status() {
            rows.push(("systemd service", desc.to_string()));
        }
        println!("{}", output::format_kv(None, &rows, color));
        let daemon_needs_restart = daemon_process_status(&context.runtime_paths.runtime_dir)
            .await
            .is_some_and(|(_, _, ready)| !ready);
        let message = if daemon_needs_restart {
            "Privileges prepared. Restart the running daemon once: `systemctl --user restart xrat-daemon.service` (standalone daemon: `xrat daemon restart`). Then run `xrat tun enable`."
        } else {
            "Privileges prepared. Run `xrat tun enable`. Repeat setup only after replacing the engine or xrat binary."
        };
        println!("{}", output::notice(message, color));
    }
    Ok(())
}
