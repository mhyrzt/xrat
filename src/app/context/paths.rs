use std::path::PathBuf;
use std::time::Duration;

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

use crate::app::AppError;
use crate::app::app_paths;
use crate::app::config::{self, AppConfig, DatabaseBackend};
use crate::cli;
use crate::db::DatabaseConnectionConfig;

#[derive(Clone)]
pub struct RuntimePaths {
    pub root_dir: PathBuf,
    pub database_config: DatabaseConnectionConfig,
    pub database_path: PathBuf,
    pub database_label: String,
    pub config_path: PathBuf,
    pub runtime_dir: PathBuf,
    pub xray_path: PathBuf,
    pub v2ray_path: PathBuf,
    pub sing_box_path: PathBuf,
}

pub fn resolve_runtime(args: &cli::Cli) -> crate::app::Result<(RuntimePaths, AppConfig)> {
    let app_paths = app_paths::ensure_layout()?;
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(|| app_paths.config_path.clone());
    app_paths::ensure_config_file(&config_path)?;
    let app_config = config::load(&config_path)?;
    let database_config =
        resolve_database_config(args, &app_config, &config_path, &app_paths.database_path)?;
    let database_path = match &database_config {
        DatabaseConnectionConfig::Sqlite { path } => path.clone(),
        DatabaseConnectionConfig::Postgres { .. } => app_paths.database_path.clone(),
    };

    Ok((
        RuntimePaths {
            root_dir: app_paths.root_dir.clone(),
            database_path,
            database_label: database_config.label(),
            database_config,
            config_path: config_path.clone(),
            runtime_dir: app_paths.root_dir.join("runtime"),
            xray_path: resolve_binary_path(
                &config_path,
                args.xray.as_ref(),
                app_config.paths.xray.as_ref(),
                "xray",
            ),
            v2ray_path: resolve_binary_path(
                &config_path,
                args.v2ray.as_ref(),
                app_config.paths.v2ray.as_ref(),
                "v2ray",
            ),
            sing_box_path: resolve_binary_path(
                &config_path,
                args.sing_box.as_ref(),
                app_config.paths.sing_box.as_ref(),
                "sing-box",
            ),
        },
        app_config,
    ))
}

pub fn resolve_config_path(args: &cli::Cli) -> crate::app::Result<PathBuf> {
    let app_paths = app_paths::ensure_layout()?;
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(|| app_paths.config_path.clone());

    app_paths::ensure_config_file(&config_path)?;
    Ok(config_path)
}

fn resolve_database_config(
    args: &cli::Cli,
    app_config: &AppConfig,
    config_path: &std::path::Path,
    default_sqlite_path: &std::path::Path,
) -> crate::app::Result<DatabaseConnectionConfig> {
    if let Some(cli_database) = &args.database {
        return Ok(DatabaseConnectionConfig::Sqlite {
            path: cli_database.clone(),
        });
    }

    match app_config.database.backend {
        DatabaseBackend::Sqlite => {
            let path = app_config
                .database
                .sqlite
                .path
                .as_deref()
                .or(app_config.paths.database.as_deref())
                .map(|path| config::resolve_config_path(config_path, path))
                .unwrap_or_else(|| default_sqlite_path.to_path_buf());

            Ok(DatabaseConnectionConfig::Sqlite { path })
        }
        DatabaseBackend::Postgres => {
            let postgres = &app_config.database.postgres;
            let user = postgres.user.resolve()?;
            let password = postgres.password.resolve()?;
            if user.is_empty() {
                return Err(AppError::MissingPostgresUser);
            }
            if postgres.db_name.is_empty() {
                return Err(AppError::MissingPostgresDatabaseName);
            }

            Ok(DatabaseConnectionConfig::Postgres {
                url: format!(
                    "postgres://{}:{}@{}:{}/{}",
                    escape(&user),
                    escape(&password),
                    postgres.host,
                    postgres.port,
                    escape(&postgres.db_name)
                ),
                max_connections: postgres.max_connections,
                min_connections: postgres.min_connections,
                connect_timeout: Duration::from_secs(postgres.connect_timeout_secs),
            })
        }
    }
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

fn escape(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}
