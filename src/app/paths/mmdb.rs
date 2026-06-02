use std::path::{Path, PathBuf};

use crate::app::config::{self, AppConfig, defaults};
use crate::app::context::RuntimePaths;

pub fn resolve_mmdb_dir(runtime_paths: &RuntimePaths, app_config: &AppConfig) -> PathBuf {
    if app_config.mmdb.dir.is_absolute() {
        return app_config.mmdb.dir.clone();
    }

    runtime_paths.root_dir.join(&app_config.mmdb.dir)
}

pub fn mmdb_path_for(
    runtime_paths: &RuntimePaths,
    app_config: &AppConfig,
    configured_path: &Path,
    file_name: &str,
) -> PathBuf {
    if configured_path.is_absolute() {
        return configured_path.to_path_buf();
    }

    if configured_path == default_mmdb_relative_path(file_name) {
        return resolve_mmdb_dir(runtime_paths, app_config).join(file_name);
    }

    config::resolve_config_path(&runtime_paths.config_path, configured_path)
}

fn default_mmdb_relative_path(file_name: &str) -> PathBuf {
    PathBuf::from(defaults::DEFAULT_MMDB_DIR).join(file_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DatabaseConnectionConfig;

    fn runtime_paths(config_path: &str) -> RuntimePaths {
        RuntimePaths {
            root_dir: "/tmp/xrat-root".into(),
            database_config: DatabaseConnectionConfig::Sqlite {
                path: "/tmp/xrat-root/db.sqlite".into(),
            },
            database_path: "/tmp/xrat-root/db.sqlite".into(),
            database_label: "/tmp/xrat-root/db.sqlite".to_string(),
            config_path: config_path.into(),
            runtime_dir: "/tmp/xrat-root/runtime".into(),
            xray_path: "xray".into(),
            v2ray_path: "v2ray".into(),
            sing_box_path: "sing-box".into(),
        }
    }

    #[test]
    fn resolves_default_mmdb_dir_from_runtime_root() {
        let runtime_paths = runtime_paths("/tmp/custom/config.toml");
        let app_config = AppConfig::default();

        assert_eq!(
            resolve_mmdb_dir(&runtime_paths, &app_config),
            PathBuf::from("/tmp/xrat-root/mmdb")
        );
    }

    #[test]
    fn resolves_default_geoip_paths_from_runtime_root_mmdb_dir() {
        let runtime_paths = runtime_paths("/tmp/custom/config.toml");
        let app_config = AppConfig::default();

        assert_eq!(
            mmdb_path_for(
                &runtime_paths,
                &app_config,
                Path::new(defaults::DEFAULT_TEST_GEOIP_COUNTRY_PATH),
                "GeoLite2-Country.mmdb"
            ),
            PathBuf::from("/tmp/xrat-root/mmdb/GeoLite2-Country.mmdb")
        );
    }

    #[test]
    fn preserves_explicit_relative_paths_as_config_relative() {
        let runtime_paths = runtime_paths("/tmp/custom/config.toml");
        let app_config = AppConfig::default();

        assert_eq!(
            mmdb_path_for(
                &runtime_paths,
                &app_config,
                Path::new("fixtures/GeoLite2-City.mmdb"),
                "GeoLite2-City.mmdb"
            ),
            PathBuf::from("/tmp/custom/fixtures/GeoLite2-City.mmdb")
        );
    }

    #[test]
    fn resolves_relative_mmdb_dir_from_runtime_root() {
        let runtime_paths = runtime_paths("/tmp/custom/config.toml");
        let app_config = AppConfig {
            mmdb: crate::app::config::MmdbSettings {
                dir: PathBuf::from("assets/mmdb"),
                ..crate::app::config::MmdbSettings::default()
            },
            ..AppConfig::default()
        };

        assert_eq!(
            resolve_mmdb_dir(&runtime_paths, &app_config),
            PathBuf::from("/tmp/xrat-root/assets/mmdb")
        );
    }
}
