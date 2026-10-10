use super::*;
use std::io::Write;
use xrat_support::process::{Command, Stdio};

pub(super) struct SpawnedRuntime {
    pub(super) pid: u32,
    pub(super) config_path: PathBuf,
}

pub(super) async fn spawn_runtime_with_ports(
    launch: &ResolvedLaunch,
    runtime_dir: &std::path::Path,
    session_id: i64,
    ports: xrat_support::readiness::RuntimeProcessPorts,
) -> crate::app::Result<SpawnedRuntime> {
    match &launch.config {
        RuntimeLaunchConfig::Xray(config) => {
            let process = xray_runtime::spawn_detached_with_ports(
                &launch.binary_path,
                runtime_dir,
                session_id,
                config,
                &launch.ready_host,
                launch.ready_port,
                Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
                ports,
            )
            .await?;
            Ok(SpawnedRuntime {
                pid: process.pid,
                config_path: process.paths.config_path,
            })
        }
        RuntimeLaunchConfig::Singbox(config) => {
            let process = singbox_runtime::spawn_detached_with_ports(
                &launch.binary_path,
                runtime_dir,
                session_id,
                config,
                &launch.ready_host,
                launch.ready_port,
                Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
                ports,
            )
            .await?;
            Ok(SpawnedRuntime {
                pid: process.pid,
                config_path: process.paths.config_path,
            })
        }
    }
}

#[cfg(test)]
pub(super) fn preflight_runtime(
    launch: &ResolvedLaunch,
    runtime_dir: &std::path::Path,
) -> crate::app::Result<()> {
    preflight_runtime_with_spawner(
        launch,
        runtime_dir,
        std::sync::Arc::new(xrat_support::process::SystemProcessSpawner),
    )
}

pub(super) fn preflight_runtime_with_spawner(
    launch: &ResolvedLaunch,
    runtime_dir: &std::path::Path,
    spawner: std::sync::Arc<dyn xrat_support::process::ProcessSpawner>,
) -> crate::app::Result<()> {
    if matches!(launch.validator, RuntimeValidator::Singbox) {
        xrat_engines::singbox::ensure_supported_binary_with_spawner(
            &launch.binary_path,
            spawner.clone(),
        )?;
    }
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
    let mut command = if matches!(launch.validator, RuntimeValidator::Singbox) {
        xrat_engines::singbox::config_check_command(&launch.binary_path, path, spawner.clone())
    } else {
        Command::with_spawner(&launch.binary_path, spawner.clone())
    };
    if let Some(directory) = xrat_support::platform::managed_core_asset_dir(&launch.binary_path) {
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
        RuntimeValidator::Singbox => {}
    }
    let output = command.stdin(Stdio::null()).output().map_err(|error| {
        AppError::XrayRuntime(
            xrat_engines::xray::runtime_process::XrayRuntimeError::Spawn(format!(
                "native config validation failed to start: {error}"
            )),
        )
    })?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if stderr.is_empty() { stdout } else { stderr };
    let hint = tun_capability_hint(&detail)
        .map(|hint| format!("; {hint}"))
        .unwrap_or_default();
    Err(AppError::InvalidArgument(format!(
        "native runtime config validation failed ({}): {}{hint}",
        output.status,
        if detail.is_empty() {
            "no diagnostic output"
        } else {
            &detail
        }
    )))
}

/// Recognize native-validation output that points at missing configuration or
/// permission problems and return an actionable hint. Preflight checks
/// (`xray run -test` and `sing-box check`) validate configuration without
/// creating the TUN interface, but permission errors can still occur when
/// probing environment or core assets.
fn tun_capability_hint(detail: &str) -> Option<&'static str> {
    let detail = detail.to_ascii_lowercase();
    let permission_denied = detail.contains("operation not permitted")
        || detail.contains("permission denied")
        || detail.contains("failed to create server");
    permission_denied.then_some(
        "if [runtime.tun] is enabled, the engine likely lacks CAP_NET_ADMIN; run `xrat tun setup`",
    )
}

#[cfg(test)]
mod tests {
    use super::tun_capability_hint;

    #[test]
    fn hint_matches_permission_failures() {
        assert!(tun_capability_hint("failed to create server > operation not permitted").is_some());
        assert!(tun_capability_hint("Permission denied").is_some());
        assert!(tun_capability_hint("unknown protocol: tun").is_none());
    }
}
