use tokio::sync::mpsc;

use crate::app::context::AppContext;
use crate::db::ImportSource;
use crate::model::Node;
use crate::tui::data::TuiData;
use crate::tui::task::{TuiTaskEvent, TuiTaskKind};

pub enum TuiImport {
    Config {
        source: ImportSource,
        node: Box<Node>,
    },
    Subscription {
        url: String,
        name: String,
    },
}

pub fn spawn_import(
    context: AppContext,
    import: TuiImport,
    include_deleted: bool,
    task_tx: &mpsc::UnboundedSender<TuiTaskEvent>,
) {
    let kind = TuiTaskKind::Import;
    let _ = task_tx.send(TuiTaskEvent::Started { kind });
    let task_tx = task_tx.clone();
    tokio::spawn(async move {
        let event = match run_import(&context, import).await {
            Ok(message) => match TuiData::load(&context, include_deleted).await {
                Ok(data) => TuiTaskEvent::Completed {
                    kind,
                    message,
                    data: Some(data),
                },
                Err(error) => TuiTaskEvent::Failed {
                    kind,
                    error: format!("import completed but reload failed: {error}"),
                    data: None,
                },
            },
            Err(error) => TuiTaskEvent::Failed {
                kind,
                error: format!("import failed: {error}"),
                data: None,
            },
        };
        let _ = task_tx.send(event);
    });
}

async fn run_import(context: &AppContext, import: TuiImport) -> crate::app::Result<String> {
    match import {
        TuiImport::Config { source, node } => {
            context
                .db
                .import_nodes(&source, std::slice::from_ref(node.as_ref()))
                .await?;
            Ok("added 1 config".to_string())
        }
        TuiImport::Subscription { url, name } => {
            let (source, nodes) = crate::app::import::load_nodes_async(&url).await?;
            let summary =
                crate::app::import::persist_nodes(&context.db, source, &nodes, Some(&name)).await?;
            Ok(format!(
                "imported {} configs into {name}",
                summary.imported_configs
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::TempDir;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    use super::{TuiImport, run_import};
    use crate::app::config::AppConfig;
    use crate::app::context::{AppContext, RuntimePaths};
    use crate::db::{Database, DatabaseConnectionConfig};

    async fn test_context() -> (TempDir, AppContext) {
        let root = tempfile::tempdir().expect("temp directory should be created");
        let database_path = root.path().join("db.sqlite");
        let database_config = DatabaseConnectionConfig::Sqlite {
            path: database_path.clone(),
        };
        let db = Database::connect(&database_config)
            .await
            .expect("database should connect");
        let runtime_paths = RuntimePaths {
            root_dir: root.path().to_path_buf(),
            database_config,
            database_path: database_path.clone(),
            database_label: database_path.display().to_string(),
            config_path: root.path().join("config.toml"),
            runtime_dir: root.path().join("runtime"),
            xray_path: PathBuf::from("xray"),
            v2ray_path: PathBuf::from("v2ray"),
            sing_box_path: PathBuf::from("sing-box"),
        };
        (
            root,
            AppContext {
                db,
                app_config: AppConfig::default(),
                runtime_paths,
            },
        )
    }

    async fn subscription_url(request_count: usize) -> String {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener should bind");
        let address = listener.local_addr().expect("listener should have address");
        tokio::spawn(async move {
            let body = "vless://uuid-123@example.com:443#One";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            for _ in 0..request_count {
                let (mut socket, _) = listener.accept().await.expect("request should connect");
                socket
                    .write_all(response.as_bytes())
                    .await
                    .expect("response should be written");
            }
        });
        format!("http://{address}/subscription")
    }

    #[tokio::test]
    async fn config_import_persists_one_config() {
        let (_root, context) = test_context().await;
        let (source, node) =
            crate::app::import::load_single_node("vless://uuid-123@example.com:443#One")
                .expect("config should parse");

        let message = run_import(
            &context,
            TuiImport::Config {
                source,
                node: Box::new(node),
            },
        )
        .await
        .expect("config import should succeed");

        assert_eq!(message, "added 1 config");
        assert_eq!(context.db.get_config_count().await.expect("count"), 1);
    }

    #[tokio::test]
    async fn subscription_import_persists_name_and_reuses_url() {
        let (_root, context) = test_context().await;
        let url = subscription_url(2).await;

        for name in ["First", "Second"] {
            run_import(
                &context,
                TuiImport::Subscription {
                    url: url.clone(),
                    name: name.to_string(),
                },
            )
            .await
            .expect("subscription import should succeed");
        }

        let subscriptions = context
            .db
            .list_subscriptions()
            .await
            .expect("subscriptions should load");
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].name.as_deref(), Some("Second"));
        assert_eq!(context.db.get_config_count().await.expect("count"), 1);
    }
}
