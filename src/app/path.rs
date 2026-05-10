use std::path::{Path, PathBuf};

use crate::app::AppError;

const APP_DIR_NAME: &str = "xrat";
const XRAT_PATH_ENV: &str = "XRAT_PATH";
const DB_FILE_NAME: &str = "db.sqlite";
const CONFIG_FILE_NAME: &str = "config.toml";
const DEFAULT_CONFIG_CONTENTS: &str = "# XRAT configuration\n\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppPaths {
    pub root_dir: PathBuf,
    pub database_path: PathBuf,
    pub config_path: PathBuf,
}

impl AppPaths {
    fn new(root_dir: PathBuf) -> Self {
        Self {
            database_path: root_dir.join(DB_FILE_NAME),
            config_path: root_dir.join(CONFIG_FILE_NAME),
            root_dir,
        }
    }
}

pub fn resolve() -> crate::app::Result<AppPaths> {
    let root_dir = resolve_root_dir_from(
        std::env::var_os(XRAT_PATH_ENV).map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )?;

    Ok(AppPaths::new(root_dir))
}

pub fn ensure_layout() -> crate::app::Result<AppPaths> {
    let paths = resolve()?;
    ensure_layout_at(&paths.root_dir)
}

pub fn ensure_config_file(config_path: &Path) -> crate::app::Result<()> {
    if let Some(parent) = config_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }

    if !config_path.exists() {
        std::fs::write(config_path, DEFAULT_CONFIG_CONTENTS)?;
    }

    Ok(())
}

fn ensure_layout_at(root_dir: &Path) -> crate::app::Result<AppPaths> {
    let paths = AppPaths::new(root_dir.to_path_buf());
    std::fs::create_dir_all(&paths.root_dir)?;
    ensure_config_file(&paths.config_path)?;

    Ok(paths)
}

fn resolve_root_dir_from(
    xrat_path: Option<PathBuf>,
    home_dir: Option<PathBuf>,
) -> crate::app::Result<PathBuf> {
    if let Some(path) = xrat_path {
        return Ok(path);
    }

    let home_dir = home_dir.ok_or(AppError::MissingHomeDirectory)?;
    Ok(home_dir.join(".config").join(APP_DIR_NAME))
}

#[cfg(test)]
mod tests;
