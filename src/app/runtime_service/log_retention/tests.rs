use super::*;
use crate::app::config::AppConfig;
use crate::app::context::RuntimePaths;
use crate::db::{Database, DatabaseConnectionConfig, RuntimeSessionInsert, RuntimeSessionStatus};

async fn context(root: &std::path::Path) -> AppContext {
    let database_path = root.join("db.sqlite");
    let database_config = DatabaseConnectionConfig::Sqlite {
        path: database_path.clone(),
    };
    AppContext {
        db: Database::connect(&database_config).await.unwrap(),
        app_config: AppConfig::default(),
        runtime_paths: RuntimePaths {
            root_dir: root.to_owned(),
            database_config,
            database_path,
            database_label: "test".into(),
            config_path: root.join("config.toml"),
            runtime_dir: root.join("runtime"),
            xray_path: "xray".into(),
            v2ray_path: "v2ray".into(),
            sing_box_path: "sing-box".into(),
        },
    }
}

async fn insert(context: &AppContext, status: RuntimeSessionStatus) -> i64 {
    context
        .db
        .insert_runtime_session(&RuntimeSessionInsert {
            config_id: None,
            status,
            socks_host: None,
            socks_port: None,
            http_host: None,
            http_port: None,
            shadowsocks_host: None,
            shadowsocks_port: None,
            process_id: None,
            failure_reason: None,
            started_at: None,
            stopped_at: None,
        })
        .await
        .unwrap()
}

#[tokio::test]
async fn retains_ten_completed_sessions_and_protects_live_and_unrelated_files() {
    let root = tempfile::tempdir().unwrap();
    let context = context(root.path()).await;
    let dir = &context.runtime_paths.runtime_dir;
    std::fs::create_dir(dir).unwrap();
    let mut protected = Vec::new();
    for status in [
        RuntimeSessionStatus::Starting,
        RuntimeSessionStatus::Running,
        RuntimeSessionStatus::Stopping,
    ] {
        let id = insert(&context, status).await;
        protected.push(format!("session-{id}.out.log"));
    }
    let mut completed = Vec::new();
    for index in 0..12 {
        let status = if index % 2 == 0 {
            RuntimeSessionStatus::Stopped
        } else {
            RuntimeSessionStatus::Failed
        };
        let id = insert(&context, status).await;
        completed.push(id);
        for suffix in ["out.log", "err.log", "singbox.out.log", "singbox.err.log"] {
            std::fs::write(dir.join(format!("session-{id}.{suffix}")), "diagnostics").unwrap();
        }
    }
    protected.extend([
        "daemon.log".into(),
        "session-9999.out.log".into(),
        format!("session-{}.json", completed[0]),
        format!("session-{}.custom.log", completed[0]),
    ]);
    for name in &protected {
        std::fs::write(dir.join(name), "keep").unwrap();
    }
    cleanup(&context).await;
    cleanup(&context).await;
    for (index, id) in completed.iter().enumerate() {
        for suffix in ["out.log", "err.log", "singbox.out.log", "singbox.err.log"] {
            assert_eq!(
                dir.join(format!("session-{id}.{suffix}")).exists(),
                index >= 2
            );
        }
    }
    for name in protected {
        assert_eq!(std::fs::read_to_string(dir.join(name)).unwrap(), "keep");
    }

    let old_path = dir.join(format!("session-{}.out.log", completed[0]));
    std::fs::create_dir(&old_path).unwrap();
    #[cfg(unix)]
    let link_path = {
        let target = root.path().join("external.log");
        std::fs::write(&target, "external diagnostics").unwrap();
        let link = dir.join(format!("session-{}.err.log", completed[0]));
        std::os::unix::fs::symlink(&target, &link).unwrap();
        link
    };
    cleanup(&context).await;
    assert!(old_path.is_dir());
    #[cfg(unix)]
    {
        assert!(link_path.is_symlink());
        assert_eq!(
            std::fs::read_to_string(link_path).unwrap(),
            "external diagnostics"
        );
    }
}

#[tokio::test]
async fn missing_or_unreadable_runtime_directory_is_best_effort() {
    let root = tempfile::tempdir().unwrap();
    let context = context(root.path()).await;
    cleanup(&context).await;
    std::fs::write(&context.runtime_paths.runtime_dir, "not a directory").unwrap();
    cleanup(&context).await;
    assert_eq!(context.db.get_runtime_session_count().await.unwrap(), 0);
}

#[test]
fn only_recognizes_managed_session_log_names() {
    for name in [
        "daemon.log",
        "session-1.json",
        "session-01.out.log",
        "session--1.out.log",
        "session-1.out.log.bak",
        "session-1.other.log",
    ] {
        assert_eq!(log_session_id(name), None, "{name}");
    }
    assert_eq!(log_session_id("session-42.singbox.err.log"), Some(42));
}
