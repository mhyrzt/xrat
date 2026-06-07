use super::*;

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
