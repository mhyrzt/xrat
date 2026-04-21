mod connection;
mod model;
mod repository;
mod schema;

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use sqlx::SqlitePool;

pub use model::{
    ConfigListFilter, ConfigRecord, ConnectionTestInsert, ConnectionTestRecord, ImportSource,
    ImportSummary, RuntimeSessionInsert, RuntimeSessionRecord, RuntimeSessionStatus, SourceKind,
    SubscriptionRecord,
};

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

    pub async fn list_configs(
        &self,
        filter: &ConfigListFilter,
    ) -> Result<Vec<ConfigRecord>, Box<dyn std::error::Error>> {
        repository::list_configs(&self.pool, filter).await
    }

    pub async fn get_config_by_id(
        &self,
        id: i64,
    ) -> Result<Option<ConfigRecord>, Box<dyn std::error::Error>> {
        repository::get_config_by_id(&self.pool, id).await
    }

    pub async fn get_selected_config(
        &self,
    ) -> Result<Option<ConfigRecord>, Box<dyn std::error::Error>> {
        repository::get_selected_config(&self.pool).await
    }

    pub async fn get_active_config(
        &self,
    ) -> Result<Option<ConfigRecord>, Box<dyn std::error::Error>> {
        repository::get_active_config(&self.pool).await
    }

    pub async fn get_subscription_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        repository::get_subscription_count(&self.pool).await
    }

    pub async fn list_subscriptions(
        &self,
    ) -> Result<Vec<SubscriptionRecord>, Box<dyn std::error::Error>> {
        repository::list_subscriptions(&self.pool).await
    }

    pub async fn get_connection_test_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        repository::get_connection_test_count(&self.pool).await
    }

    pub async fn insert_connection_test(
        &self,
        test: &ConnectionTestInsert,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        repository::insert_connection_test(&self.pool, test).await
    }

    pub async fn list_connection_tests(
        &self,
        config_id: i64,
    ) -> Result<Vec<ConnectionTestRecord>, Box<dyn std::error::Error>> {
        repository::list_connection_tests(&self.pool, config_id).await
    }

    pub async fn get_latest_connection_test(
        &self,
        config_id: i64,
    ) -> Result<Option<ConnectionTestRecord>, Box<dyn std::error::Error>> {
        repository::get_latest_connection_test(&self.pool, config_id).await
    }

    pub async fn get_runtime_session_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        repository::get_runtime_session_count(&self.pool).await
    }

    pub async fn insert_runtime_session(
        &self,
        session: &RuntimeSessionInsert,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        repository::insert_runtime_session(&self.pool, session).await
    }

    pub async fn get_latest_runtime_session(
        &self,
    ) -> Result<Option<RuntimeSessionRecord>, Box<dyn std::error::Error>> {
        repository::get_latest_runtime_session(&self.pool).await
    }

    pub async fn get_running_runtime_session(
        &self,
    ) -> Result<Option<RuntimeSessionRecord>, Box<dyn std::error::Error>> {
        repository::get_running_runtime_session(&self.pool).await
    }

    pub async fn update_runtime_session_state(
        &self,
        session_id: i64,
        status: RuntimeSessionStatus,
        process_id: Option<i64>,
        mixed_port: Option<i64>,
        started_at: Option<&str>,
        stopped_at: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        repository::update_runtime_session_state(
            &self.pool, session_id, status, process_id, mixed_port, started_at, stopped_at,
        )
        .await
    }

    pub async fn mark_runtime_session_stopped(
        &self,
        session_id: i64,
        stopped_at: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        repository::mark_runtime_session_stopped(&self.pool, session_id, stopped_at).await
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

    pub async fn set_selected_config(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        repository::set_selected_config(&self.pool, id).await
    }

    pub async fn set_active_config(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        repository::set_active_config(&self.pool, id).await
    }

    pub async fn set_config_enabled(
        &self,
        id: i64,
        is_enabled: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        repository::set_config_enabled(&self.pool, id, is_enabled).await
    }

    pub async fn soft_delete_config(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        repository::soft_delete_config(&self.pool, id).await
    }

    pub async fn restore_config(&self, id: i64) -> Result<(), Box<dyn std::error::Error>> {
        repository::restore_config(&self.pool, id).await
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
    use super::{
        ConfigListFilter, ConnectionTestInsert, Database, ImportSource, RuntimeSessionInsert,
        RuntimeSessionStatus, SourceKind, test_database_path,
    };
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

        let subscriptions = db.list_subscriptions().await.expect("subscriptions");
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].source_kind, "url");
        assert_eq!(subscriptions[0].config_count, 1);

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

    #[tokio::test]
    async fn lists_configs_and_hides_deleted_by_default() {
        let db_path = test_database_path("xrat-list");
        let db = Database::connect(&db_path).await.expect("db should open");
        let source = ImportSource {
            kind: SourceKind::File,
            value: "sample.txt".to_string(),
            name: None,
        };
        let first = test_node("first");
        let first_key = first.dedup_key_string();
        let mut second = test_node("second");
        second.address = "second.example.com".to_string();

        db.import_nodes(&source, &[first, second])
            .await
            .expect("import should succeed");
        db.mark_deleted(&first_key)
            .await
            .expect("delete should succeed");

        let visible = db
            .list_configs(&ConfigListFilter::default())
            .await
            .expect("list should succeed");
        let all = db
            .list_configs(&ConfigListFilter {
                include_deleted: true,
                ..ConfigListFilter::default()
            })
            .await
            .expect("list with deleted should succeed");
        let deleted_only = db
            .list_configs(&ConfigListFilter {
                include_deleted: true,
                only_deleted: true,
                ..ConfigListFilter::default()
            })
            .await
            .expect("deleted-only list should succeed");

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].address, "second.example.com");
        assert_eq!(all.len(), 2);
        assert!(all.iter().any(|config| config.is_deleted));
        assert_eq!(deleted_only.len(), 1);
        assert!(deleted_only[0].is_deleted);
        assert_eq!(deleted_only[0].address, "example.com");

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn config_selection_and_activation_are_exclusive() {
        let db_path = test_database_path("xrat-config-state");
        let db = Database::connect(&db_path).await.expect("db should open");
        let source = ImportSource {
            kind: SourceKind::File,
            value: "sample.txt".to_string(),
            name: None,
        };
        let first = test_node("first");
        let mut second = test_node("second");
        second.address = "second.example.com".to_string();

        db.import_nodes(&source, &[first, second])
            .await
            .expect("import should succeed");

        let configs = db
            .list_configs(&ConfigListFilter::default())
            .await
            .expect("list should succeed");
        let first_id = configs[0].id;
        let second_id = configs[1].id;

        db.set_selected_config(first_id)
            .await
            .expect("select first should succeed");
        db.set_selected_config(second_id)
            .await
            .expect("select second should succeed");
        db.set_active_config(first_id)
            .await
            .expect("activate first should succeed");
        db.set_active_config(second_id)
            .await
            .expect("activate second should succeed");

        let selected = db
            .get_selected_config()
            .await
            .expect("selected query should succeed")
            .expect("selected config should exist");
        let active = db
            .get_active_config()
            .await
            .expect("active query should succeed")
            .expect("active config should exist");
        let configs = db
            .list_configs(&ConfigListFilter {
                include_deleted: true,
                ..ConfigListFilter::default()
            })
            .await
            .expect("list should succeed");

        assert_eq!(selected.id, second_id);
        assert_eq!(active.id, second_id);
        assert_eq!(
            configs.iter().filter(|config| config.is_selected).count(),
            1
        );
        assert_eq!(configs.iter().filter(|config| config.is_active).count(), 1);

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn disabling_and_restoring_configs_updates_normal_visibility() {
        let db_path = test_database_path("xrat-config-visibility");
        let db = Database::connect(&db_path).await.expect("db should open");
        let source = ImportSource {
            kind: SourceKind::File,
            value: "sample.txt".to_string(),
            name: None,
        };

        db.import_nodes(&source, &[test_node("first")])
            .await
            .expect("import should succeed");

        let config = db
            .list_configs(&ConfigListFilter::default())
            .await
            .expect("list should succeed")
            .into_iter()
            .next()
            .expect("config should exist");

        db.set_selected_config(config.id)
            .await
            .expect("select should succeed");
        db.set_active_config(config.id)
            .await
            .expect("activate should succeed");
        db.set_config_enabled(config.id, false)
            .await
            .expect("disable should succeed");

        let disabled = db
            .get_config_by_id(config.id)
            .await
            .expect("query should succeed")
            .expect("config should still exist");
        assert!(!disabled.is_enabled);
        assert!(!disabled.is_selected);
        assert!(!disabled.is_active);

        db.soft_delete_config(config.id)
            .await
            .expect("soft delete should succeed");
        assert!(
            db.list_configs(&ConfigListFilter::default())
                .await
                .expect("visible list should succeed")
                .is_empty()
        );

        db.restore_config(config.id)
            .await
            .expect("restore should succeed");
        db.set_config_enabled(config.id, true)
            .await
            .expect("re-enable should succeed");

        let restored = db
            .get_config_by_id(config.id)
            .await
            .expect("query should succeed")
            .expect("config should still exist");
        assert!(!restored.is_deleted);
        assert!(restored.deleted_at.is_none());
        assert!(restored.is_enabled);

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn stores_and_reads_connection_test_history() {
        let db_path = test_database_path("xrat-connection-tests");
        let db = Database::connect(&db_path).await.expect("db should open");
        let source = ImportSource {
            kind: SourceKind::File,
            value: "sample.txt".to_string(),
            name: None,
        };

        db.import_nodes(&source, &[test_node("first")])
            .await
            .expect("import should succeed");

        let config = db
            .list_configs(&ConfigListFilter::default())
            .await
            .expect("list should succeed")
            .into_iter()
            .next()
            .expect("config should exist");

        db.insert_connection_test(&ConnectionTestInsert {
            config_id: config.id,
            tcp_ok: Some(false),
            tcp_ms: None,
            real_delay_ok: None,
            real_delay_ms: None,
            failure_kind: Some("timeout".to_string()),
            failure_reason: Some("tcp handshake timed out".to_string()),
        })
        .await
        .expect("first test insert should succeed");

        db.insert_connection_test(&ConnectionTestInsert {
            config_id: config.id,
            tcp_ok: Some(true),
            tcp_ms: Some(120),
            real_delay_ok: Some(true),
            real_delay_ms: Some(240),
            failure_kind: None,
            failure_reason: None,
        })
        .await
        .expect("second test insert should succeed");

        let tests = db
            .list_connection_tests(config.id)
            .await
            .expect("history should load");
        let latest = db
            .get_latest_connection_test(config.id)
            .await
            .expect("latest should load")
            .expect("latest record should exist");

        assert_eq!(db.get_connection_test_count().await.expect("count"), 2);
        assert_eq!(tests.len(), 2);
        assert_eq!(latest.config_id, config.id);
        assert_eq!(latest.tcp_ok, Some(true));
        assert_eq!(latest.tcp_ms, Some(120));
        assert_eq!(latest.real_delay_ok, Some(true));
        assert_eq!(latest.real_delay_ms, Some(240));
        assert_eq!(latest.failure_kind, None);

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn stores_and_updates_runtime_sessions() {
        let db_path = test_database_path("xrat-runtime-sessions");
        let db = Database::connect(&db_path).await.expect("db should open");
        let source = ImportSource {
            kind: SourceKind::File,
            value: "sample.txt".to_string(),
            name: None,
        };

        db.import_nodes(&source, &[test_node("first")])
            .await
            .expect("import should succeed");

        let config = db
            .list_configs(&ConfigListFilter::default())
            .await
            .expect("list should succeed")
            .into_iter()
            .next()
            .expect("config should exist");

        let session_id = db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(config.id),
                status: RuntimeSessionStatus::Starting,
                mixed_port: Some(10808),
                process_id: None,
                started_at: Some("2025-01-01T10:00:00Z".to_string()),
                stopped_at: None,
            })
            .await
            .expect("runtime session insert should succeed");

        let running = db
            .get_running_runtime_session()
            .await
            .expect("running session query should succeed")
            .expect("running session should exist");
        assert_eq!(running.id, session_id);
        assert_eq!(running.status, RuntimeSessionStatus::Starting);
        assert_eq!(running.mixed_port, Some(10808));

        db.update_runtime_session_state(
            session_id,
            RuntimeSessionStatus::Running,
            Some(4242),
            Some(10808),
            None,
            None,
        )
        .await
        .expect("runtime session update should succeed");

        db.mark_runtime_session_stopped(session_id, Some("2025-01-01T10:05:00Z"))
            .await
            .expect("runtime session stop should succeed");

        let latest = db
            .get_latest_runtime_session()
            .await
            .expect("latest session query should succeed")
            .expect("latest session should exist");

        assert_eq!(db.get_runtime_session_count().await.expect("count"), 1);
        assert_eq!(latest.id, session_id);
        assert_eq!(latest.status, RuntimeSessionStatus::Stopped);
        assert_eq!(latest.process_id, Some(4242));
        assert_eq!(latest.mixed_port, Some(10808));
        assert_eq!(latest.stopped_at.as_deref(), Some("2025-01-01T10:05:00Z"));
        assert!(
            db.get_running_runtime_session()
                .await
                .expect("running query should succeed")
                .is_none()
        );

        let _ = std::fs::remove_file(db_path);
    }
}
