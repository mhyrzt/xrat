use crate::app::config::{ConfigEditSession, SettingValue, TunSettings, TunSplitMode};
use crate::app::context::AppContext;
use crate::app::daemon::ipc::{self, TunStatePayload};
use crate::app::runtime_service::RuntimeService;
use crate::app::{AppError, Result};

pub(crate) async fn apply(
    context: &mut AppContext,
    enabled: Option<bool>,
    standalone_owner: bool,
) -> Result<TunStatePayload> {
    #[cfg(unix)]
    {
        let socket = ipc::default_socket_path(&context.runtime_paths.runtime_dir);
        if super::runtime_control::tui_uses_daemon(&socket).await? {
            let ping = ipc::ping_daemon(&socket).await?;
            if !ping.payload.is_some_and(|payload| payload.live_tun) {
                return Err(AppError::InvalidArgument(
                    "The running daemon predates live TUN toggles. Restart it once after upgrading, then retry. The current connection was kept.".into(),
                ));
            }
            let response = ipc::runtime_tun_daemon(
                &socket,
                enabled,
                context.runtime_paths.config_path.clone(),
            )
            .await?;
            if !response.ok {
                return Err(AppError::InvalidArgument(response.message));
            }
            let payload = response
                .payload
                .ok_or_else(|| AppError::InvalidArgument("Daemon returned no TUN state".into()))?;
            reload(context)?;
            return Ok(payload);
        }
    }
    if !standalone_owner && RuntimeService::new(context).status().await?.pid_running {
        return Err(AppError::InvalidArgument(
            "The active runtime belongs to a standalone TUI. Press U there to toggle TUN, or use a daemon for CLI control. The current connection was kept.".into(),
        ));
    }
    super::tun::apply(context, enabled).await
}

pub(crate) async fn mutate_tun_settings<F>(
    context: &mut AppContext,
    standalone_owner: bool,
    mutate: F,
) -> Result<TunStatePayload>
where
    F: FnOnce(&mut TunSettings) -> Result<()>,
{
    let config_path = context.runtime_paths.config_path.clone();
    let mut session = ConfigEditSession::open(&config_path).map_err(AppError::InvalidArgument)?;
    let mut config: crate::app::config::AppConfig = toml::from_str(&session.original_contents)?;
    let previous_tun = config.runtime.tun.clone();
    mutate(&mut config.runtime.tun)?;

    if let Some(error) = crate::app::commands::validate::validate_app_config(&config).first() {
        return Err(AppError::InvalidArgument(error.clone()));
    }

    apply_tun_settings_to_session(&mut session, &config.runtime.tun)?;
    session.save().map_err(|error| {
        AppError::InvalidArgument(format!("Could not save TUN settings: {error}"))
    })?;

    let target_enabled = config.runtime.tun.enabled;
    match apply(context, Some(target_enabled), standalone_owner).await {
        Ok(payload) => Ok(payload),
        Err(error) => {
            let _ = restore_tun_settings_on_disk(&config_path, &previous_tun);
            Err(error)
        }
    }
}

fn apply_tun_settings_to_session(session: &mut ConfigEditSession, tun: &TunSettings) -> Result<()> {
    for (path, value) in [
        ("runtime.tun.enabled", SettingValue::Bool(tun.enabled)),
        (
            "runtime.tun.split_mode",
            SettingValue::Text(tun.split_mode.as_str().to_string()),
        ),
        (
            "runtime.tun.blacklist",
            SettingValue::List(tun.blacklist.clone()),
        ),
        (
            "runtime.tun.whitelist",
            SettingValue::List(tun.whitelist.clone()),
        ),
    ] {
        let setting = session
            .settings
            .iter_mut()
            .find(|setting| setting.path == path)
            .ok_or_else(|| {
                AppError::InvalidArgument(format!("TUN setting `{path}` was not found"))
            })?;
        setting.value = value;
    }
    Ok(())
}

fn restore_tun_settings_on_disk(
    config_path: &std::path::Path,
    previous_tun: &TunSettings,
) -> Result<()> {
    let mut session = ConfigEditSession::open(config_path).map_err(AppError::InvalidArgument)?;
    apply_tun_settings_to_session(&mut session, previous_tun)?;
    session.save().map(|_| ()).map_err(|error| {
        AppError::InvalidArgument(format!("Could not restore TUN settings: {error}"))
    })
}

pub(crate) fn reload(context: &mut AppContext) -> Result<()> {
    let contents = std::fs::read_to_string(&context.runtime_paths.config_path)?;
    let config: crate::app::config::AppConfig = toml::from_str(&contents)?;
    context.app_config.runtime = config.runtime;
    context.app_config.dns = config.dns;
    context.app_config.routing = config.routing;
    for (configured, binary) in [
        (&config.paths.xray, &mut context.runtime_paths.xray_path),
        (
            &config.paths.sing_box,
            &mut context.runtime_paths.sing_box_path,
        ),
        (&config.paths.v2ray, &mut context.runtime_paths.v2ray_path),
    ] {
        if let Some(path) = configured {
            *binary =
                crate::app::config::resolve_config_path(&context.runtime_paths.config_path, path);
        }
    }
    Ok(())
}

pub(crate) fn split_summary(state: &TunStatePayload) -> String {
    match TunSplitMode::from_config_str(&state.split_mode).unwrap_or_default() {
        TunSplitMode::All => "split: all".to_string(),
        TunSplitMode::Blacklist => format!("split: blacklist ({})", state.blacklist_count),
        TunSplitMode::Whitelist => format!("split: whitelist ({})", state.whitelist_count),
    }
}

pub(crate) fn message(state: &TunStatePayload) -> String {
    let mode = if state.enabled { "enabled" } else { "disabled" };
    let split = split_summary(state);
    if let Some(config) = &state.active_config_ref {
        if state.active {
            format!(
                "TUN {mode} on config {config} via {} ({}, {split}).",
                state.engine,
                state.interface.as_deref().unwrap_or("unknown interface")
            )
        } else {
            format!("TUN {mode} ({split}); config {config} is connected in proxy mode.")
        }
    } else if state.enabled {
        format!(
            "TUN enabled ({split}); it will apply on the next connection. Run `xrat tun status` to check privileges."
        )
    } else {
        format!("TUN disabled ({split}); future connections use proxy mode.")
    }
}
