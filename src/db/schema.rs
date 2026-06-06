use std::borrow::Cow;

use sqlx::migrate::MigrateError;
use sqlx::migrate::{Migration, MigrationType};
use sqlx::{PgPool, SqlitePool};

use crate::db::DbError;

static SQLITE_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite");
static POSTGRES_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/postgres");

pub async fn init_sqlite(pool: &SqlitePool) -> crate::db::Result<()> {
    repair_migration_19_checksum_sqlite(pool).await?;
    SQLITE_MIGRATOR
        .run(pool)
        .await
        .map_err(|err| migration_error("SQLite", err))?;
    Ok(())
}

pub async fn init_postgres(pool: &PgPool) -> crate::db::Result<()> {
    repair_migration_19_checksum_postgres(pool).await?;
    POSTGRES_MIGRATOR
        .run(pool)
        .await
        .map_err(|err| migration_error("PostgreSQL", err))?;
    Ok(())
}

const MIGRATION_19_VERSION: i64 = 19;

fn migration_19_checksum(sql: &'static str) -> Vec<u8> {
    Migration::new(
        MIGRATION_19_VERSION,
        "migration-19".into(),
        MigrationType::ReversibleUp,
        Cow::Borrowed(sql),
        false,
    )
    .checksum
    .into_owned()
}

fn current_migration_19_checksum_sqlite() -> Vec<u8> {
    migration_19_checksum(include_str!(
        "../../migrations/sqlite/0019_add_config_subscription_refs.sql"
    ))
}

fn current_migration_19_checksum_postgres() -> Vec<u8> {
    migration_19_checksum(include_str!(
        "../../migrations/postgres/0019_add_config_subscription_refs.sql"
    ))
}

async fn repair_migration_19_checksum_sqlite(pool: &SqlitePool) -> crate::db::Result<()> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    if exists.is_none() {
        return Ok(());
    }

    let checksum: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT checksum FROM _sqlx_migrations WHERE version = ? AND success = 1",
    )
    .bind(MIGRATION_19_VERSION)
    .fetch_optional(pool)
    .await?;
    let current_checksum = current_migration_19_checksum_sqlite();
    if checksum.as_deref() == Some(current_checksum.as_slice()) {
        return Ok(());
    }

    sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
        .bind(current_checksum)
        .bind(MIGRATION_19_VERSION)
        .execute(pool)
        .await?;
    Ok(())
}

async fn repair_migration_19_checksum_postgres(pool: &PgPool) -> crate::db::Result<()> {
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = '_sqlx_migrations')",
    )
    .fetch_optional(pool)
    .await?;
    if exists != Some(true) {
        return Ok(());
    }

    let checksum: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT checksum FROM _sqlx_migrations WHERE version = $1 AND success = true",
    )
    .bind(MIGRATION_19_VERSION)
    .fetch_optional(pool)
    .await?;
    let current_checksum = current_migration_19_checksum_postgres();
    if checksum.as_deref() == Some(current_checksum.as_slice()) {
        return Ok(());
    }

    sqlx::query("UPDATE _sqlx_migrations SET checksum = $1 WHERE version = $2")
        .bind(current_checksum)
        .bind(MIGRATION_19_VERSION)
        .execute(pool)
        .await?;
    Ok(())
}

/// Translate a raw sqlx migration failure into an actionable error. Migrations
/// run lazily on the first command after an upgrade, so a bare sqlx message here
/// is easy to misread as unrelated corruption. We attach the migration version,
/// the likely cause, and concrete recovery guidance.
fn migration_error(backend: &str, err: MigrateError) -> DbError {
    let guidance = match &err {
        MigrateError::VersionMismatch(version) => format!(
            "migration {version} was already applied but its checksum no longer matches this build. \
             A previously shipped migration file was most likely edited after release. \
             Policy: never edit an applied migration; always add a new ordered migration. \
             To recover, restore the original {version} migration (e.g. reinstall the matching \
             release) or reset the database from a backup."
        ),
        MigrateError::Dirty(version) => format!(
            "migration {version} is partially applied (dirty state). \
             Inspect the `_sqlx_migrations` table, finish or revert migration {version} by hand, \
             and remove its row before retrying."
        ),
        MigrateError::VersionMissing(version) => format!(
            "migration {version} is recorded as applied in the database but is missing from this build. \
             This usually means a downgrade to an older xrat. Upgrade back to a build that includes \
             migration {version}."
        ),
        MigrateError::ExecuteMigration(source, version) => format!(
            "migration {version} failed while executing against existing data: {source}. \
             Review migration {version} and the current database contents."
        ),
        other => format!("{other}"),
    };

    DbError::MigrationFailed(format!("{backend} database migration failed: {guidance}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::collections::BTreeMap;

    const SQLITE_MIGRATION_19_LEGACY_SQL: &str = "-- User-facing stable short refs for configs and subscriptions.\n\
-- SQLite cannot add a NOT NULL UNIQUE column via ALTER, so the column is added\n\
-- nullable, backfilled with random 12-char hex, and a UNIQUE index is created.\n\
-- The application layer always sets a ref on insert.\n\
ALTER TABLE configs ADD COLUMN ref TEXT;\n\
ALTER TABLE subscriptions ADD COLUMN ref TEXT;\n\
\n\
UPDATE configs SET ref = lower(hex(randomblob(6)))\n\
WHERE ref IS NULL;\n\
UPDATE subscriptions SET ref = lower(hex(randomblob(6)))\n\
WHERE ref IS NULL;\n\
\n\
CREATE UNIQUE INDEX idx_configs_ref ON configs (ref);\n\
CREATE UNIQUE INDEX idx_subscriptions_ref ON subscriptions (ref);\n";

    fn legacy_migration_19_checksum_sqlite() -> Vec<u8> {
        migration_19_checksum(SQLITE_MIGRATION_19_LEGACY_SQL)
    }

    /// Map each up migration's version to its hex-encoded checksum. Down
    /// migrations are not tracked in `_sqlx_migrations`, so they are skipped.
    fn migration_checksums(migrator: &sqlx::migrate::Migrator) -> BTreeMap<String, String> {
        migrator
            .iter()
            .filter(|migration| !migration.migration_type.is_down_migration())
            .map(|migration| {
                let hex = migration
                    .checksum
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                (migration.version.to_string(), hex)
            })
            .collect()
    }

    const MIGRATION_MANIFEST_PATH: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/migrations/checksums.json");

    async fn memory_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite should connect")
    }

    #[tokio::test]
    async fn sqlite_migrations_apply_cleanly() {
        let pool = memory_pool().await;
        init_sqlite(&pool)
            .await
            .expect("migrations should apply cleanly on a fresh database");
    }

    #[tokio::test]
    async fn sqlite_migrations_are_idempotent() {
        let pool = memory_pool().await;
        init_sqlite(&pool).await.expect("first migration run");
        init_sqlite(&pool)
            .await
            .expect("re-running the migrator must be a no-op, not an error");
    }

    #[tokio::test]
    async fn repairs_legacy_migration_19_checksum_before_running() {
        let pool = memory_pool().await;
        sqlx::query(
            "CREATE TABLE _sqlx_migrations (version INTEGER PRIMARY KEY, description TEXT NOT NULL, installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, success BOOLEAN NOT NULL, checksum BLOB NOT NULL, execution_time BIGINT NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("create migrations table");

        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)",
        )
        .bind(MIGRATION_19_VERSION)
        .bind("migration 19")
        .bind(legacy_migration_19_checksum_sqlite())
        .execute(&pool)
        .await
        .expect("seed legacy checksum");

        repair_migration_19_checksum_sqlite(&pool)
            .await
            .expect("repair should succeed");

        let checksum: Vec<u8> =
            sqlx::query_scalar("SELECT checksum FROM _sqlx_migrations WHERE version = ?")
                .bind(MIGRATION_19_VERSION)
                .fetch_one(&pool)
                .await
                .expect("fetch repaired checksum");
        assert_eq!(checksum, current_migration_19_checksum_sqlite());
    }

    /// Guard against the root cause of the migration-19 incident: editing a
    /// migration file after it was applied or released changes its checksum and
    /// breaks every database that already ran it. We commit a checksum manifest
    /// and verify migration files still match it. A change here is intentional
    /// only when no database has applied the migration yet.
    #[test]
    fn migration_files_match_committed_checksum_manifest() {
        let mut current = BTreeMap::new();
        current.insert("sqlite".to_string(), migration_checksums(&SQLITE_MIGRATOR));
        current.insert(
            "postgres".to_string(),
            migration_checksums(&POSTGRES_MIGRATOR),
        );

        if std::env::var_os("UPDATE_MIGRATION_MANIFEST").is_some() {
            let mut json = serde_json::to_string_pretty(&current).expect("serialize manifest");
            json.push('\n');
            std::fs::write(MIGRATION_MANIFEST_PATH, json).expect("write migration manifest");
            return;
        }

        let raw = std::fs::read_to_string(MIGRATION_MANIFEST_PATH).unwrap_or_else(|err| {
            panic!(
                "missing migration checksum manifest at {MIGRATION_MANIFEST_PATH}: {err}. \
                 Generate it with `UPDATE_MIGRATION_MANIFEST=1 cargo test \
                 migration_files_match_committed_checksum_manifest`."
            )
        });
        let committed: BTreeMap<String, BTreeMap<String, String>> =
            serde_json::from_str(&raw).expect("parse migration manifest");

        let mut problems = Vec::new();
        for backend in ["sqlite", "postgres"] {
            let expected = committed.get(backend).cloned().unwrap_or_default();
            let actual = current.get(backend).cloned().unwrap_or_default();
            for (version, checksum) in &actual {
                match expected.get(version) {
                    Some(committed_checksum) if committed_checksum == checksum => {}
                    Some(_) => problems.push(format!(
                        "{backend} migration {version} checksum changed (the file was edited)"
                    )),
                    None => problems.push(format!(
                        "{backend} migration {version} is new and not yet in the manifest"
                    )),
                }
            }
            for version in expected.keys() {
                if !actual.contains_key(version) {
                    problems.push(format!(
                        "{backend} migration {version} is in the manifest but the file is gone"
                    ));
                }
            }
        }

        assert!(
            problems.is_empty(),
            "migration checksum manifest is out of date:\n  {}\n\n\
             Never edit a migration that has been applied or released; add a new ordered \
             migration instead. Only if this change is intentional and no database has applied \
             the affected migration, regenerate the manifest with \
             `UPDATE_MIGRATION_MANIFEST=1 cargo test migration_files_match_committed_checksum_manifest`.",
            problems.join("\n  ")
        );
    }
}
