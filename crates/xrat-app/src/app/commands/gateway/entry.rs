use super::actions::run_with_engine_path;
use crate::app::{
    AppError, Result,
    config::{self, AppConfig},
};
use crate::cli::{Cli, GatewayAction, GatewayArgs};

pub async fn run(cli: &Cli, args: &GatewayArgs) -> Result<()> {
    let path = match &cli.config {
        Some(path) => path.clone(),
        None => crate::app::app_paths::resolve()?.config_path,
    };
    let config = if matches!(args.action, GatewayAction::Remove(_)) {
        AppConfig::default()
    } else if path.exists() {
        config::load(&path)?
    } else if cli.config.is_some() {
        return Err(AppError::InvalidArgument(format!(
            "configuration file does not exist: {}",
            path.display()
        )));
    } else {
        AppConfig::default()
    };
    let engine_path = match config.runtime.engine.as_str() {
        "xray" => cli
            .xray
            .clone()
            .or_else(|| {
                config
                    .paths
                    .xray
                    .as_ref()
                    .map(|value| config::resolve_config_path(&path, value))
            })
            .unwrap_or_else(|| "xray".into()),
        "sing-box" => cli
            .sing_box
            .clone()
            .or_else(|| {
                config
                    .paths
                    .sing_box
                    .as_ref()
                    .map(|value| config::resolve_config_path(&path, value))
            })
            .unwrap_or_else(|| "sing-box".into()),
        _ => std::path::PathBuf::new(),
    };
    let runtime_dir = crate::app::app_paths::resolve()
        .ok()
        .map(|paths| paths.root_dir.join("runtime"));
    run_with_engine_path(&config, args, Some(&engine_path), runtime_dir.as_deref()).await
}

pub async fn run_with_config(
    config: &AppConfig,
    args: &GatewayArgs,
    runtime_dir: &std::path::Path,
) -> Result<()> {
    run_with_engine_path(config, args, None, Some(runtime_dir)).await
}
