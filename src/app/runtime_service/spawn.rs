use super::*;
use std::io::Write;
use std::process::{Command, Stdio};

pub(super) struct SpawnedRuntime {
    pub(super) pid: u32,
    pub(super) config_path: PathBuf,
}

pub(super) async fn spawn_runtime(
    launch: &ResolvedLaunch,
    runtime_dir: &std::path::Path,
    session_id: i64,
) -> crate::app::Result<SpawnedRuntime> {
    match &launch.config {
        RuntimeLaunchConfig::Xray(config) => {
            let process = xray_runtime::spawn_detached(
                &launch.binary_path,
                runtime_dir,
                session_id,
                config,
                &launch.ready_host,
                launch.ready_port,
                Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
            )
            .await?;
            Ok(SpawnedRuntime {
                pid: process.pid,
                config_path: process.paths.config_path,
            })
        }
        RuntimeLaunchConfig::Singbox(config) => {
            let process = singbox_runtime::spawn_detached(
                &launch.binary_path,
                runtime_dir,
                session_id,
                config,
                &launch.ready_host,
                launch.ready_port,
                Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
            )
            .await?;
            Ok(SpawnedRuntime {
                pid: process.pid,
                config_path: process.paths.config_path,
            })
        }
    }
}

pub(super) fn preflight_runtime(
    launch: &ResolvedLaunch,
    runtime_dir: &std::path::Path,
) -> crate::app::Result<()> {
    std::fs::create_dir_all(runtime_dir)?;
    let mut temporary = tempfile::Builder::new()
        .prefix(".xrat-preflight-")
        .suffix(".json")
        .tempfile_in(runtime_dir)?;
    match &launch.config {
        RuntimeLaunchConfig::Xray(config) => {
            temporary.write_all(serde_json::to_string_pretty(config)?.as_bytes())?;
        }
        RuntimeLaunchConfig::Singbox(config) => {
            temporary.write_all(serde_json::to_string_pretty(config)?.as_bytes())?;
        }
    }
    temporary.as_file_mut().flush()?;
    let path = temporary.path();
    let mut command = Command::new(&launch.binary_path);
    if let Some(directory) = crate::support::platform::managed_core_asset_dir(&launch.binary_path) {
        let variable = match launch.validator {
            RuntimeValidator::V2ray => "V2RAY_LOCATION_ASSET",
            RuntimeValidator::Xray => "XRAY_LOCATION_ASSET",
            RuntimeValidator::Singbox => "",
        };
        if !variable.is_empty() {
            command.env(variable, directory);
        }
    }
    match launch.validator {
        RuntimeValidator::Xray => {
            command.arg("run").arg("-test").arg("-c").arg(path);
        }
        RuntimeValidator::V2ray => {
            command.arg("test").arg("-c").arg(path);
        }
        RuntimeValidator::Singbox => {
            command.arg("check").arg("-c").arg(path);
        }
    }
    let output = command.stdin(Stdio::null()).output().map_err(|error| {
        AppError::XraySpawn(format!("native config validation failed to start: {error}"))
    })?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if stderr.is_empty() { stdout } else { stderr };
    Err(AppError::InvalidArgument(format!(
        "native runtime config validation failed ({}): {}",
        output.status,
        if detail.is_empty() {
            "no diagnostic output"
        } else {
            &detail
        }
    )))
}
