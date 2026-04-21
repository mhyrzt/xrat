use std::path::PathBuf;

use crate::app::path;
use crate::cli;
use crate::db::Database;

#[derive(Clone)]
pub struct AppContext {
    pub db: Database,
    pub runtime_paths: RuntimePaths,
}

#[derive(Clone)]
pub struct RuntimePaths {
    pub database_path: PathBuf,
    pub config_path: PathBuf,
}

impl AppContext {
    pub async fn build(args: &cli::Cli) -> Result<Self, Box<dyn std::error::Error>> {
        let runtime_paths = resolve_runtime_paths(args)?;
        let db = Database::connect(&runtime_paths.database_path).await?;

        Ok(Self { db, runtime_paths })
    }
}

fn resolve_runtime_paths(args: &cli::Cli) -> Result<RuntimePaths, Box<dyn std::error::Error>> {
    let app_paths = path::ensure_layout()?;
    let database_path = args
        .database
        .clone()
        .unwrap_or_else(|| app_paths.database_path.clone());
    let config_path = args
        .config
        .clone()
        .unwrap_or_else(|| app_paths.config_path.clone());

    path::ensure_config_file(&config_path)?;

    Ok(RuntimePaths {
        database_path,
        config_path,
    })
}
