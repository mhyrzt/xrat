use std::path::PathBuf;
use std::time::Duration;

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

use crate::app::AppError;
use crate::app::config::{self, AppConfig, DatabaseBackend};
use crate::app::path;
use crate::cli;
use crate::db::{Database, DatabaseConnectionConfig};

#[derive(Clone)]
pub struct AppContext {
    pub db: Database,
    pub app_config: AppConfig,
    pub runtime_paths: RuntimePaths,
}

#[derive(Clone)]
pub struct RuntimePaths {
    pub database_config: DatabaseConnectionConfig,
    pub database_path: PathBuf,
    pub database_label: String,
    pub config_path: PathBuf,
    pub xray_path: PathBuf,
    pub v2ray_path: PathBuf,
}

impl AppContext {
    pub async fn build(args: &cli::Cli) -> crate::app::Result<Self> {
        let (runtime_paths, app_config) = resolve_runtime(args)?;
        let db = Database::connect(&runtime_paths.database_config).await?;

        Ok(Self {
            db,
            app_config,
            runtime_paths,
        })
    }
}

fn resolve_runtime(args: &cli::Cli) -> crate::app::Result<(RuntimePaths, AppConfig)> {
    let app_paths = path::ensure_layout()?;
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(|| app_paths.config_path.clone());

    path::ensure_config_file(&config_path)?;
    let app_config = config::load(&config_path)?;
    let database_config =
        resolve_database_config(args, &app_config, &config_path, &app_paths.database_path)?;
    let database_path = match &database_config {
        DatabaseConnectionConfig::Sqlite { path } => path.clone(),
        DatabaseConnectionConfig::Postgres { .. } => app_paths.database_path.clone(),
    };
    let database_label = database_config.label();
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
            database_config,
            database_path,
            database_label,
            config_path,
            xray_path,
            v2ray_path,
        },
        app_config,
    ))
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
            let user = utf8_percent_encode(&user, NON_ALPHANUMERIC).to_string();
            let password = utf8_percent_encode(&password, NON_ALPHANUMERIC).to_string();
            let db_name = utf8_percent_encode(&postgres.db_name, NON_ALPHANUMERIC).to_string();
            let url = format!(
                "postgres://{user}:{password}@{}:{}/{db_name}",
                postgres.host, postgres.port
            );

            Ok(DatabaseConnectionConfig::Postgres {
                url,
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
    fn resolves_postgres_database_from_config_file() {
        let root_dir = std::env::temp_dir().join(format!(
            "xrat-runtime-config-postgres-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        let config_path = root_dir.join("config.toml");
        std::fs::create_dir_all(&root_dir).expect("temp dir should be created");
        std::fs::write(
            &config_path,
            "[database]\nbackend = \"postgres\"\n\n[database.postgres]\nuser = \"xrat user\"\npassword = \"secret/pass\"\nhost = \"db.local\"\nport = 5544\ndb_name = \"xrat db\"\n",
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

        match runtime_paths.database_config {
            crate::db::DatabaseConnectionConfig::Postgres { url, .. } => {
                assert_eq!(
                    url,
                    "postgres://xrat%20user:secret%2Fpass@db.local:5544/xrat%20db"
                );
            }
            crate::db::DatabaseConnectionConfig::Sqlite { .. } => {
                panic!("expected postgres config")
            }
        }
        assert_eq!(
            runtime_paths.database_label,
            "postgres://xrat%20user:<redacted>@db.local:5544/xrat%20db"
        );

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
