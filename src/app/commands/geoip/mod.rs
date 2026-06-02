use std::path::{Path, PathBuf};

use crate::app::context::AppContext;
use crate::app::paths::mmdb;
use crate::cli::{GeoIpAction, GeoIpArgs};

mod download;
mod edition;
mod path;
mod status;
mod update;

pub async fn run(context: &AppContext, args: &GeoIpArgs) -> crate::app::Result<()> {
    match &args.action {
        GeoIpAction::Download(args) => download::run(context, args).await,
        GeoIpAction::Update(args) => update::run(context, args).await,
        GeoIpAction::Path(args) => path::run(context, args),
        GeoIpAction::Status(args) => status::run(context, args),
    }
}

fn resolve_mmdb_target_dir(context: &AppContext, output_override: Option<&PathBuf>) -> PathBuf {
    output_override
        .cloned()
        .unwrap_or_else(|| mmdb::resolve_mmdb_dir(&context.runtime_paths, &context.app_config))
}

fn ensure_mmdb_target_dir(dir: &Path) -> crate::app::Result<()> {
    std::fs::create_dir_all(dir)?;
    Ok(())
}

fn mmdb_file_path(dir: &Path, edition: edition::MmdbEdition) -> PathBuf {
    dir.join(edition.file_name())
}

fn mmdb_file_name(edition: edition::MmdbEdition) -> &'static str {
    edition.file_name()
}

const SUPPORTED_EDITIONS: [edition::MmdbEdition; 3] = edition::SUPPORTED_EDITIONS;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::AppConfig;
    use crate::app::context::RuntimePaths;
    use crate::db::{Database, DatabaseConnectionConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolves_default_target_dir_from_mmdb_config() {
        let context = test_context();

        assert_eq!(
            resolve_mmdb_target_dir(&context, None),
            context.runtime_paths.root_dir.join("mmdb")
        );
    }

    #[test]
    fn resolves_override_target_dir_verbatim() {
        let context = test_context();

        assert_eq!(
            resolve_mmdb_target_dir(&context, Some(&PathBuf::from("./tmp/mmdb"))),
            PathBuf::from("./tmp/mmdb")
        );
    }

    fn test_context() -> AppContext {
        let root = std::env::temp_dir().join(format!(
            "xrat-geoip-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("root should be created");
        let database_config = DatabaseConnectionConfig::Sqlite {
            path: root.join("db.sqlite"),
        };
        let runtime = tokio::runtime::Runtime::new().expect("runtime should build");
        let db = runtime
            .block_on(Database::connect(&database_config))
            .expect("database should connect");

        AppContext {
            db,
            app_config: AppConfig::default(),
            runtime_paths: RuntimePaths {
                root_dir: root.clone(),
                database_config,
                database_path: root.join("db.sqlite"),
                database_label: root.join("db.sqlite").display().to_string(),
                config_path: root.join("config.toml"),
                runtime_dir: root.join("runtime"),
                xray_path: "xray".into(),
                v2ray_path: "v2ray".into(),
                sing_box_path: "sing-box".into(),
            },
        }
    }
}
