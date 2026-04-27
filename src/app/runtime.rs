use std::path::PathBuf;

use crate::app::config::{self, AppConfig};
use crate::app::path;
use crate::cli;
use crate::db::Database;

#[derive(Clone)]
pub struct AppContext {
    pub db: Database,
    pub app_config: AppConfig,
    pub runtime_paths: RuntimePaths,
}

#[derive(Clone)]
pub struct RuntimePaths {
    pub database_path: PathBuf,
    pub config_path: PathBuf,
    pub xray_path: PathBuf,
    pub v2ray_path: PathBuf,
}

impl AppContext {
    pub async fn build(args: &cli::Cli) -> Result<Self, Box<dyn std::error::Error>> {
        let (runtime_paths, app_config) = resolve_runtime(args)?;
        let db = Database::connect(&runtime_paths.database_path).await?;

        Ok(Self {
            db,
            app_config,
            runtime_paths,
        })
    }
}

fn resolve_runtime(
    args: &cli::Cli,
) -> Result<(RuntimePaths, AppConfig), Box<dyn std::error::Error>> {
    let app_paths = path::ensure_layout()?;
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(|| app_paths.config_path.clone());

    path::ensure_config_file(&config_path)?;
    let app_config = config::load(&config_path)?;
    let database_path = args
        .database
        .clone()
        .or_else(|| {
            app_config
                .paths
                .database
                .as_deref()
                .map(|path| config::resolve_config_path(&config_path, path))
        })
        .unwrap_or_else(|| app_paths.database_path.clone());
    let xray_path = resolve_binary_path(
        &config_path,
        args.xray.as_ref(),
        app_config.paths.xray.as_ref(),
        "xray",
    );
    let v2ray_path = resolve_binary_path(
        &config_path,
        args.v2ray.as_ref(),
        app_config.paths.v2ray.as_ref(),
        "v2ray",
    );

    Ok((
        RuntimePaths {
            database_path,
            config_path,
            xray_path,
            v2ray_path,
        },
        app_config,
    ))
}

fn resolve_binary_path(
    config_path: &std::path::Path,
    cli_path: Option<&PathBuf>,
    config_value: Option<&PathBuf>,
    fallback: &str,
) -> PathBuf {
    cli_path
        .cloned()
        .or_else(|| config_value.map(|path| config::resolve_config_path(config_path, path)))
        .unwrap_or_else(|| PathBuf::from(fallback))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use clap::Parser;

    use super::resolve_runtime;
    use crate::cli::Cli;

    #[test]
    fn resolves_database_from_config_file() {
        let root_dir = std::env::temp_dir().join(format!(
            "xrat-runtime-config-db-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        let config_path = root_dir.join("config.toml");
        std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
        std::fs::write(&config_path, "[paths]\ndatabase = \"state/db.sqlite\"\n")
            .expect("config should be written");

        let cli = Cli::parse_from([
            "xrat",
            "--config",
            config_path.to_str().unwrap(),
            "list",
            "configs",
        ]);
        let (runtime_paths, _) = resolve_runtime(&cli).expect("runtime paths should resolve");

        assert_eq!(
            runtime_paths.database_path,
            root_dir.join("state/db.sqlite")
        );
        assert_eq!(runtime_paths.xray_path, PathBuf::from("xray"));
        assert_eq!(runtime_paths.v2ray_path, PathBuf::from("v2ray"));

        let _ = std::fs::remove_file(config_path);
        let _ = std::fs::remove_dir(root_dir);
    }

    #[test]
    fn cli_database_overrides_config_database() {
        let root_dir = std::env::temp_dir().join(format!(
            "xrat-runtime-cli-db-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        let config_path = root_dir.join("config.toml");
        let cli_database = root_dir.join("override.sqlite");
        std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
        std::fs::write(&config_path, "[paths]\ndatabase = \"state/db.sqlite\"\n")
            .expect("config should be written");

        let cli = Cli::parse_from([
            "xrat",
            "--config",
            config_path.to_str().unwrap(),
            "--database",
            cli_database.to_str().unwrap(),
            "list",
            "configs",
        ]);
        let (runtime_paths, _) = resolve_runtime(&cli).expect("runtime paths should resolve");

        assert_eq!(runtime_paths.database_path, cli_database);

        let _ = std::fs::remove_file(config_path);
        let _ = std::fs::remove_dir(root_dir);
    }

    #[test]
    fn resolves_binary_paths_from_config_file() {
        let root_dir = std::env::temp_dir().join(format!(
            "xrat-runtime-config-binaries-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        let config_path = root_dir.join("config.toml");
        std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
        std::fs::write(
            &config_path,
            "[paths]\nxray = \"bin/xray\"\nv2ray = \"/opt/v2ray/v2ray\"\n",
        )
        .expect("config should be written");

        let cli = Cli::parse_from([
            "xrat",
            "--config",
            config_path.to_str().unwrap(),
            "list",
            "configs",
        ]);
        let (runtime_paths, _) = resolve_runtime(&cli).expect("runtime paths should resolve");

        assert_eq!(runtime_paths.xray_path, root_dir.join("bin/xray"));
        assert_eq!(runtime_paths.v2ray_path, PathBuf::from("/opt/v2ray/v2ray"));

        let _ = std::fs::remove_file(config_path);
        let _ = std::fs::remove_dir(root_dir);
    }

    #[test]
    fn cli_binary_paths_override_config_file() {
        let root_dir = std::env::temp_dir().join(format!(
            "xrat-runtime-cli-binaries-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        let config_path = root_dir.join("config.toml");
        std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
        std::fs::write(
            &config_path,
            "[paths]\nxray = \"bin/xray\"\nv2ray = \"bin/v2ray\"\n",
        )
        .expect("config should be written");

        let cli = Cli::parse_from([
            "xrat",
            "--config",
            config_path.to_str().unwrap(),
            "--xray",
            "/custom/xray",
            "--v2ray",
            "/custom/v2ray",
            "list",
            "configs",
        ]);
        let (runtime_paths, _) = resolve_runtime(&cli).expect("runtime paths should resolve");

        assert_eq!(runtime_paths.xray_path, PathBuf::from("/custom/xray"));
        assert_eq!(runtime_paths.v2ray_path, PathBuf::from("/custom/v2ray"));

        let _ = std::fs::remove_file(config_path);
        let _ = std::fs::remove_dir(root_dir);
    }
}
