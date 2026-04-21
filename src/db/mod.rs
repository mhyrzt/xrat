mod connection;
mod models;
mod repository;
mod schema;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::SqlitePool;

pub use models::{ImportSource, ImportSummary, SourceKind};

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn connect(database_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let pool = connection::connect(database_path).await?;
        Ok(Self { pool })
    }

    pub async fn import_nodes(
        &self,
        source: &ImportSource,
        nodes: &[crate::model::Node],
    ) -> Result<ImportSummary, Box<dyn std::error::Error>> {
        repository::import_nodes(&self.pool, source, nodes).await
    }

    pub async fn get_config_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        repository::get_config_count(&self.pool).await
    }

    pub async fn get_subscription_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        repository::get_subscription_count(&self.pool).await
    }

    pub async fn get_connection_test_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        repository::get_connection_test_count(&self.pool).await
    }

    pub async fn get_runtime_session_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        repository::get_runtime_session_count(&self.pool).await
    }

    pub async fn get_config_flags(
        &self,
        dedup_key: &str,
    ) -> Result<(bool, bool, bool, bool), Box<dyn std::error::Error>> {
        repository::get_config_flags(&self.pool, dedup_key).await
    }

    pub async fn mark_deleted(&self, dedup_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        repository::mark_deleted(&self.pool, dedup_key).await
    }
}

pub fn test_database_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}.sqlite"))
}

#[cfg(test)]
mod tests {
    use super::{Database, ImportSource, SourceKind, test_database_path};
    use crate::model::{Node, Protocol};

    fn test_node(name: &str) -> Node {
        Node {
            protocol: Protocol::Vless,
            address: "example.com".to_string(),
            port: 443,
            username: None,
            uuid: Some("uuid-123".to_string()),
            password: None,
            method: None,
            network: "ws".to_string(),
            tls: Some("tls".to_string()),
            sni: Some("cdn.example.com".to_string()),
            host: Some("cdn.example.com".to_string()),
            path: Some("/socket".to_string()),
            name: Some(name.to_string()),
        }
    }

    #[tokio::test]
    async fn imports_nodes_and_creates_subscription() {
        let db_path = test_database_path("xrat-import");
        let db = Database::connect(&db_path).await.expect("db should open");
        let source = ImportSource {
            kind: SourceKind::Url,
            value: "https://example.com/sub".to_string(),
            name: Some("Example".to_string()),
        };

        let summary = db
            .import_nodes(&source, &[test_node("first")])
            .await
            .expect("import should succeed");

        assert_eq!(summary.imported_configs, 1);
        assert_eq!(summary.total_configs, 1);
        assert_eq!(db.get_subscription_count().await.expect("count"), 1);
        assert_eq!(db.get_config_count().await.expect("count"), 1);
        assert_eq!(db.get_connection_test_count().await.expect("count"), 0);
        assert_eq!(db.get_runtime_session_count().await.expect("count"), 0);

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn upsert_revives_soft_deleted_config_without_creating_duplicates() {
        let db_path = test_database_path("xrat-upsert");
        let db = Database::connect(&db_path).await.expect("db should open");
        let source = ImportSource {
            kind: SourceKind::File,
            value: "sample.txt".to_string(),
            name: None,
        };
        let node = test_node("first");
        let dedup_key = node.dedup_key_string();

        db.import_nodes(&source, std::slice::from_ref(&node))
            .await
            .expect("first import should succeed");
        db.mark_deleted(&dedup_key)
            .await
            .expect("soft delete should succeed");
        db.import_nodes(&source, &[test_node("updated")])
            .await
            .expect("second import should succeed");

        assert_eq!(db.get_config_count().await.expect("count"), 1);
        assert_eq!(
            db.get_config_flags(&dedup_key).await.expect("flags"),
            (false, true, false, false)
        );

        let _ = std::fs::remove_file(db_path);
    }
}
