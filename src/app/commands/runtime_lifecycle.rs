use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::app::runtime::AppContext;
use crate::db::{RuntimeSessionRecord, RuntimeSessionStatus};
use crate::xray::runtime as xray_runtime;

const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActiveSessionState {
    None,
    Running(RuntimeSessionRecord),
    Stale(RuntimeSessionRecord),
}

pub async fn active_session_state(context: &AppContext) -> crate::app::Result<ActiveSessionState> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        return Ok(ActiveSessionState::None);
    };

    if runtime_session_is_alive(&session) {
        return Ok(ActiveSessionState::Running(session));
    }

    mark_session_stale(context, &session).await?;
    Ok(ActiveSessionState::Stale(session))
}

pub async fn stop_active_session(context: &AppContext) -> crate::app::Result<bool> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        context.db.clear_active_config().await?;
        return Ok(false);
    };

    context
        .db
        .update_runtime_session_state(
            session.id,
            RuntimeSessionStatus::Stopping,
            None,
            None,
            None,
            None,
        )
        .await?;

    if let Some(pid) = session.process_id {
        let outcome = xray_runtime::terminate_process_gracefully(pid, SHUTDOWN_TIMEOUT)?;
        tracing::info!(
            session_id = session.id,
            pid,
            outcome = ?outcome,
            "runtime process termination completed"
        );
    } else {
        tracing::warn!(
            session_id = session.id,
            "runtime session has no saved process id"
        );
    }

    context
        .db
        .mark_runtime_session_stopped(session.id, Some(&now_string()))
        .await?;
    context.db.clear_active_config().await?;
    Ok(true)
}

pub async fn mark_session_stale(
    context: &AppContext,
    session: &RuntimeSessionRecord,
) -> crate::app::Result<()> {
    let terminal_status = match session.status {
        RuntimeSessionStatus::Stopping => RuntimeSessionStatus::Stopped,
        RuntimeSessionStatus::Starting | RuntimeSessionStatus::Running => {
            RuntimeSessionStatus::Failed
        }
        RuntimeSessionStatus::Stopped | RuntimeSessionStatus::Failed => return Ok(()),
    };

    context
        .db
        .update_runtime_session_state(
            session.id,
            terminal_status,
            None,
            None,
            None,
            Some(&now_string()),
        )
        .await?;
    context.db.clear_active_config().await?;
    Ok(())
}

pub fn runtime_session_is_alive(session: &RuntimeSessionRecord) -> bool {
    session
        .process_id
        .map(xray_runtime::process_is_running)
        .unwrap_or(false)
}

pub fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::AppConfig;
    use crate::app::runtime::{AppContext, RuntimePaths};
    use crate::db::{
        Database, DatabaseConnectionConfig, ImportSource, RuntimeSessionInsert, SourceKind,
    };
    use crate::model::{Node, Protocol};

    #[tokio::test]
    async fn marks_running_session_with_dead_pid_as_failed() {
        let context = test_context().await;
        let summary = context
            .db
            .import_nodes(&test_source(), &[test_node()])
            .await
            .expect("node should import");
        let config = context
            .db
            .list_configs(&Default::default())
            .await
            .expect("configs should load")
            .into_iter()
            .next()
            .expect("config should exist");
        assert_eq!(summary.imported_configs, 1);
        context
            .db
            .set_active_config(config.id)
            .await
            .expect("active config should be set");
        context
            .db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(config.id),
                status: RuntimeSessionStatus::Running,
                mixed_port: Some(1080),
                process_id: Some(0),
                started_at: Some("1".to_string()),
                stopped_at: None,
            })
            .await
            .expect("session should insert");

        let state = active_session_state(&context)
            .await
            .expect("session state should resolve");

        assert!(matches!(state, ActiveSessionState::Stale(_)));
        assert_eq!(
            context
                .db
                .get_latest_runtime_session()
                .await
                .expect("latest should load")
                .expect("latest should exist")
                .status,
            RuntimeSessionStatus::Failed
        );
        assert!(
            context
                .db
                .get_active_config()
                .await
                .expect("active should load")
                .is_none()
        );
    }

    async fn test_context() -> AppContext {
        let root = std::env::temp_dir().join(format!(
            "xrat-runtime-lifecycle-{}",
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
            },
        }
    }

    fn test_source() -> ImportSource {
        ImportSource {
            kind: SourceKind::RawText,
            value: "test".to_string(),
            name: Some("test".to_string()),
        }
    }

    fn test_node() -> Node {
        Node {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("00000000-0000-0000-0000-000000000000".to_string()),
            password: None,
            method: None,
            network: "tcp".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("example.com".to_string()),
            host: None,
            path: None,
            name: Some("test".to_string()),
            raw_config: "vless://test".to_string(),
        }
    }
}
