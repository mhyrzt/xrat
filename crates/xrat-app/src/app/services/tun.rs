use crate::app::config::{ConfigEditSession, SettingValue};
use crate::app::context::AppContext;
use crate::app::daemon::ipc::TunStatePayload;
use crate::app::runtime_service::{
    ConnectRequest, RuntimeService, RuntimeStatusSnapshot, tun_ownership,
};
use crate::app::{AppError, Result};
use xrat_support::readiness::RuntimeProcessPorts;

pub(crate) fn capture_state(
    context: &AppContext,
    snapshot: &RuntimeStatusSnapshot,
    ports: &RuntimeProcessPorts,
) -> TunStatePayload {
    let record = tun_ownership::load_ownership(&context.runtime_paths.runtime_dir);
    let owned = record.filter(|record| {
        snapshot.pid_running
            && snapshot
                .session
                .as_ref()
                .is_some_and(|session| session.id == record.session_id)
            && ports
                .tun
                .inspect_interface(&record.interface_name)
                .ok()
                .flatten()
                .is_some_and(|info| {
                    info.is_tun && record.ifindex == Some(info.ifindex) && info.ifindex > 0
                })
    });
    let tun = &context.app_config.runtime.tun;
    TunStatePayload {
        enabled: tun.enabled,
        active: owned.is_some(),
        engine: owned
            .as_ref()
            .map(|record| record.engine.clone())
            .unwrap_or_else(|| context.app_config.runtime.engine.clone()),
        interface: owned.map(|record| record.interface_name),
        active_config_ref: snapshot
            .active_config
            .as_ref()
            .filter(|_| snapshot.pid_running)
            .map(|config| config.r#ref.clone()),
        session_id: snapshot.session.as_ref().map(|session| session.id),
        split_mode: tun.split_mode.as_str().to_string(),
        blacklist_count: tun.blacklist.len(),
        whitelist_count: tun.whitelist.len(),
    }
}

pub(crate) fn check_engine(context: &AppContext, ports: &RuntimeProcessPorts) -> Result<()> {
    if !cfg!(target_os = "linux") {
        return Err(AppError::InvalidArgument(
            "Managed TUN is supported on Linux only.".into(),
        ));
    }
    let binary = match context.app_config.runtime.engine.as_str() {
        "xray" => {
            crate::app::services::runtime_tuning::ensure_xray_tun_supported_with_spawner(
                &context.runtime_paths.xray_path,
                ports.spawner.clone(),
            )?;
            &context.runtime_paths.xray_path
        }
        "sing-box" => &context.runtime_paths.sing_box_path,
        engine => {
            return Err(AppError::InvalidArgument(format!(
                "Cannot enable TUN with {engine}. Select Xray or sing-box in settings."
            )));
        }
    };
    let resolved = crate::app::tun_privileges::resolve_executable(binary).ok_or_else(|| {
        AppError::InvalidArgument(format!(
            "TUN engine not found at {}. Run `xrat install {}`.",
            binary.display(),
            context.app_config.runtime.engine,
        ))
    })?;
    crate::app::tun_privileges::ensure_engine_capability_with_spawner(
        &resolved,
        ports.spawner.clone(),
    )
}

pub(crate) async fn apply(
    context: &mut AppContext,
    enabled: Option<bool>,
) -> Result<TunStatePayload> {
    let process_ready = crate::app::tun_privileges::inspect_process_privileges(std::process::id())
        .is_some_and(|(_, ready)| ready);
    apply_with_ports(
        context,
        enabled,
        RuntimeProcessPorts::default(),
        process_ready,
    )
    .await
}

pub(crate) async fn apply_with_ports(
    context: &mut AppContext,
    requested: Option<bool>,
    ports: RuntimeProcessPorts,
    process_ready: bool,
) -> Result<TunStatePayload> {
    let mut session = ConfigEditSession::open(&context.runtime_paths.config_path)
        .map_err(AppError::InvalidArgument)?;
    let mut config: crate::app::config::AppConfig = toml::from_str(&session.original_contents)?;
    let enabled = requested.unwrap_or(!config.runtime.tun.enabled);
    config.runtime.tun.enabled = enabled;
    if let Some(error) = crate::app::commands::validate::validate_app_config(&config).first() {
        return Err(AppError::InvalidArgument(error.clone()));
    }
    let setting = session
        .settings
        .iter_mut()
        .find(|setting| setting.path == "runtime.tun.enabled")
        .ok_or_else(|| AppError::InvalidArgument("TUN setting was not found".into()))?;
    setting.value = SettingValue::Bool(enabled);

    let mut candidate = context.clone();
    candidate.app_config.runtime.tun = config.runtime.tun;
    candidate.app_config.runtime.engine = config.runtime.engine;
    candidate.app_config.runtime.xray_compatibility = config.runtime.xray_compatibility;
    candidate.app_config.dns = config.dns;
    candidate.app_config.routing = config.routing;
    for (configured, binary) in [
        (&config.paths.xray, &mut candidate.runtime_paths.xray_path),
        (
            &config.paths.sing_box,
            &mut candidate.runtime_paths.sing_box_path,
        ),
        (&config.paths.v2ray, &mut candidate.runtime_paths.v2ray_path),
    ] {
        if let Some(path) = configured {
            *binary =
                crate::app::config::resolve_config_path(&context.runtime_paths.config_path, path);
        }
    }
    if enabled {
        check_engine(&candidate, &ports)?;
    }
    let before = RuntimeService::with_process_ports(context, ports.clone())
        .status()
        .await?;
    let previous_capture = capture_state(context, &before, &ports);
    let active_config = before.active_config.as_ref().filter(|_| before.pid_running);
    let mut previous = context.clone();
    let previous_ownership = tun_ownership::load_ownership(&context.runtime_paths.runtime_dir)
        .filter(|record| {
            before
                .session
                .as_ref()
                .is_some_and(|session| session.id == record.session_id)
        });
    previous.app_config.runtime.tun.enabled = previous_ownership.is_some();
    if let Some(record) = previous_ownership {
        previous.app_config.runtime.engine = record.engine;
        previous.app_config.runtime.tun.interface_name = record.interface_name;
    }
    previous.app_config.runtime.replace_active_session = true;
    let mut replaced = false;

    if let Some(active) = active_config {
        if (enabled || previous.app_config.runtime.tun.enabled) && !process_ready {
            return Err(AppError::InvalidArgument(
                "The running runtime owner lacks TUN privileges. Run `xrat tun setup`. Restart a standalone TUI once, or restart the daemon once (`systemctl --user restart xrat-daemon.service`, or `xrat daemon restart` for a standalone daemon), then retry. The current connection was kept.".into(),
            ));
        }
        if previous_capture.active != enabled
            || context.app_config.runtime != candidate.app_config.runtime
            || context.app_config.dns != candidate.app_config.dns
            || context.app_config.routing != candidate.app_config.routing
        {
            let replace_active = candidate.app_config.runtime.replace_active_session;
            candidate.app_config.runtime.replace_active_session = true;
            RuntimeService::with_process_ports(&candidate, ports.clone())
                .with_rollback_context(&previous)
                .connect(ConnectRequest {
                    config_id: active.id,
                })
                .await?;
            candidate.app_config.runtime.replace_active_session = replace_active;
            replaced = true;
        }
    }
    let after = match RuntimeService::with_process_ports(&candidate, ports.clone())
        .status()
        .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            if let Some(active) = active_config.filter(|_| replaced) {
                return Err(restore(&previous, active.id, ports, error).await);
            }
            return Err(error);
        }
    };
    let outcome = capture_state(&candidate, &after, &ports);
    if let Some(active) = active_config.filter(|_| replaced)
        && outcome.active != enabled
    {
        let error = AppError::InvalidArgument(
            "TUN change did not produce the expected interface state".into(),
        );
        return Err(restore(&previous, active.id, ports, error).await);
    }
    if let Err(error) = session.save() {
        let error = AppError::InvalidArgument(format!("Could not save TUN setting: {error}"));
        if let Some(active) = active_config.filter(|_| replaced) {
            return Err(restore(&previous, active.id, ports, error).await);
        }
        return Err(error);
    }
    *context = candidate;
    crate::app::events::record(
        &context.db,
        crate::app::events::LEVEL_INFO,
        crate::app::events::SOURCE_RUNTIME,
        "tun_mode_changed",
        if enabled {
            "TUN enabled"
        } else {
            "TUN disabled"
        },
        active_config.map(|config| config.id),
        outcome.session_id,
        Some(format!("enabled={enabled}; active={}", outcome.active)),
    )
    .await;
    Ok(outcome)
}

async fn restore(
    previous: &AppContext,
    config_id: xrat_model::ConfigId,
    ports: RuntimeProcessPorts,
    error: AppError,
) -> AppError {
    let mut previous = previous.clone();
    previous.app_config.runtime.replace_active_session = true;
    match RuntimeService::with_process_ports(&previous, ports)
        .connect(ConnectRequest { config_id })
        .await
    {
        Ok(_) => {
            AppError::InvalidArgument(format!("{error}; previous mode and connection restored"))
        }
        Err(rollback) => AppError::InvalidArgument(format!("{error}; rollback failed: {rollback}")),
    }
}
