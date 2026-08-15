mod paths;

use crate::app::config::AppConfig;
use crate::cli;
use crate::db::Database;

pub use paths::{RuntimePaths, resolve_config_path};

#[derive(Clone)]
pub struct AppContext {
    pub db: Database,
    pub app_config: AppConfig,
    pub runtime_paths: RuntimePaths,
}

impl AppContext {
    pub async fn build(args: &cli::Cli) -> crate::app::Result<Self> {
        let (runtime_paths, app_config) = paths::resolve_runtime(args)?;
        let db = Database::connect(&runtime_paths.database_config).await?;
        Ok(Self {
            db,
            app_config,
            runtime_paths,
        })
    }
}

#[cfg(test)]
mod tests;
