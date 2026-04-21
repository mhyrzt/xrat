use std::path::{Path, PathBuf};

const APP_DIR_NAME: &str = "xrat";
const XRAT_PATH_ENV: &str = "XRAT_PATH";
const DB_FILE_NAME: &str = "db.sqlite";
const CONFIG_FILE_NAME: &str = "Config.toml";
const DEFAULT_CONFIG_CONTENTS: &str = "# XRAT configuration\n";

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

pub fn resolve() -> Result<AppPaths, Box<dyn std::error::Error>> {
    let root_dir = resolve_root_dir_from(
        std::env::var_os(XRAT_PATH_ENV).map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )?;

    Ok(AppPaths::new(root_dir))
}

pub fn ensure_layout() -> Result<AppPaths, Box<dyn std::error::Error>> {
    let paths = resolve()?;
    ensure_layout_at(&paths.root_dir)
}

pub fn ensure_config_file(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

fn ensure_layout_at(root_dir: &Path) -> Result<AppPaths, Box<dyn std::error::Error>> {
    let paths = AppPaths::new(root_dir.to_path_buf());
    std::fs::create_dir_all(&paths.root_dir)?;
    ensure_config_file(&paths.config_path)?;

    Ok(paths)
}

fn resolve_root_dir_from(
    xrat_path: Option<PathBuf>,
    home_dir: Option<PathBuf>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(path) = xrat_path {
        return Ok(path);
    }

    let home_dir = home_dir.ok_or("could not determine XRAT home directory")?;
    Ok(home_dir.join(".config").join(APP_DIR_NAME))
}

#[cfg(test)]
mod tests {
    use super::{AppPaths, ensure_config_file, ensure_layout_at, resolve_root_dir_from};

    #[test]
    fn uses_xrat_path_when_present() {
        let resolved =
            resolve_root_dir_from(Some("/tmp/custom-xrat".into()), Some("/home/tester".into()))
                .expect("path should resolve");

        assert_eq!(resolved, std::path::PathBuf::from("/tmp/custom-xrat"));
    }

    #[test]
    fn falls_back_to_home_config_directory() {
        let resolved =
            resolve_root_dir_from(None, Some("/home/tester".into())).expect("path should resolve");

        assert_eq!(
            resolved,
            std::path::PathBuf::from("/home/tester/.config/xrat")
        );
    }

    #[test]
    fn ensures_layout_and_creates_default_config_file() {
        let root_dir = std::env::temp_dir().join(format!(
            "xrat-app-layout-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));

        let paths = ensure_layout_at(&root_dir).expect("layout should be created");

        assert_eq!(
            paths,
            AppPaths {
                root_dir: root_dir.clone(),
                database_path: root_dir.join("db.sqlite"),
                config_path: root_dir.join("Config.toml"),
            }
        );
        assert!(paths.root_dir.is_dir());
        assert!(paths.config_path.is_file());

        let config = std::fs::read_to_string(&paths.config_path).expect("config should exist");
        assert!(config.contains("XRAT configuration"));

        let _ = std::fs::remove_file(paths.config_path);
        let _ = std::fs::remove_dir(paths.root_dir);
    }

    #[test]
    fn ensures_overridden_config_file_parent_and_file_exist() {
        let root_dir = std::env::temp_dir().join(format!(
            "xrat-config-override-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        let config_path = root_dir.join("nested").join("custom.toml");

        ensure_config_file(&config_path).expect("config file should be created");

        assert!(config_path.is_file());
        let config = std::fs::read_to_string(&config_path).expect("config should exist");
        assert!(config.contains("XRAT configuration"));

        let _ = std::fs::remove_file(&config_path);
        let _ = std::fs::remove_dir(root_dir.join("nested"));
        let _ = std::fs::remove_dir(root_dir);
    }
}
