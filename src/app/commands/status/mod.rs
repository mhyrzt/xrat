use crate::app::context::AppContext;
use crate::app::daemon::ipc;
use crate::cli::StatusArgs;

mod display;

pub async fn run(context: &AppContext, args: &StatusArgs) -> crate::app::Result<()> {
    let socket_path = ipc::default_socket_path(&context.runtime_paths.runtime_dir);
    match ipc::runtime_status_daemon(&socket_path).await {
        Ok(response) => display::print_daemon_status(response, args.json),
        Err(err) if ipc::daemon_unreachable(&err) => {
            Err(crate::app::AppError::InvalidArgument(format!(
                "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                socket_path.display()
            )))
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppError;
    use crate::app::config::AppConfig;
    use crate::app::context::RuntimePaths;
    use crate::db::{Database, DatabaseConnectionConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn status_returns_daemon_unreachable_hint() {
        let context = test_context("status-daemon-hint").await;
        let err = run(&context, &StatusArgs { json: false })
            .await
            .expect_err("status should require daemon reachability");
        match err {
            AppError::InvalidArgument(message) => {
                assert!(message.contains("xrat daemon start"));
                assert!(message.contains("daemon is not running"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    async fn test_context(prefix: &str) -> AppContext {
        let root = std::env::temp_dir().join(format!(
            "xrat-command-{prefix}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time should be valid")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("root should be created");
        let database_config = DatabaseConnectionConfig::Sqlite {
            path: root.join("db.sqlite"),
        };
        let db = Database::connect(&database_config)
            .await
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
