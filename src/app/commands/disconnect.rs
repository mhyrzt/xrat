use crate::app::daemon::server;
use crate::app::runtime::AppContext;
use crate::cli::DisconnectArgs;

pub async fn run(context: &AppContext, args: &DisconnectArgs) -> crate::app::Result<()> {
    let socket_path = server::default_socket_path(&context.runtime_paths.runtime_dir);
    let result = match server::runtime_disconnect_daemon(&socket_path).await {
        Ok(response) => {
            if !response.ok {
                return Err(crate::app::AppError::InvalidArgument(response.message));
            }
            let payload = response.payload.ok_or_else(|| {
                crate::app::AppError::InvalidArgument(
                    "daemon disconnect response missing payload".to_string(),
                )
            })?;
            crate::app::runtime_service::DisconnectResult {
                stopped_session: payload.stopped_session,
            }
        }
        Err(err) if server::daemon_unreachable(&err) => {
            return Err(crate::app::AppError::InvalidArgument(format!(
                "daemon is not running. Start it with `xrat daemon start` (socket: {})",
                socket_path.display()
            )));
        }
        Err(err) => return Err(err),
    };

    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "stopped_session": result.stopped_session,
                "message": if result.stopped_session {
                    "Disconnected active runtime session"
                } else {
                    "No active runtime session"
                },
            }))?
        );
        return Ok(());
    }

    if result.stopped_session {
        println!("Disconnected active runtime session");
    } else {
        println!("No active runtime session");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppError;
    use crate::app::config::AppConfig;
    use crate::app::runtime::RuntimePaths;
    use crate::db::{Database, DatabaseConnectionConfig};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn disconnect_returns_daemon_unreachable_hint() {
        let context = test_context("disconnect-daemon-hint").await;
        let err = run(&context, &DisconnectArgs { json: false })
            .await
            .expect_err("disconnect should require daemon reachability");
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
