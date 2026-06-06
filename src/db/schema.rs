use std::borrow::Cow;

use sqlx::migrate::MigrateError;
use sqlx::migrate::{Migration, MigrationType};
use sqlx::{PgPool, SqlitePool};

use crate::db::DbError;

static SQLITE_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/sqlite");
static POSTGRES_MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations/postgres");

pub async fn init_sqlite(pool: &SqlitePool) -> crate::db::Result<()> {
    repair_checksums_sqlite(pool).await?;
    SQLITE_MIGRATOR
        .run(pool)
        .await
        .map_err(|err| migration_error("SQLite", err))?;
    Ok(())
}

pub async fn init_postgres(pool: &PgPool) -> crate::db::Result<()> {
    repair_checksums_postgres(pool).await?;
    POSTGRES_MIGRATOR
        .run(pool)
        .await
        .map_err(|err| migration_error("PostgreSQL", err))?;
    Ok(())
}

/// Normalize migration SQL so cosmetic reformatting (whitespace, line wrapping,
/// `--` line comments) does not change the value we compare. This assumes
/// migrations do not contain `--` inside a string literal, which holds for this
/// project.
fn normalize_sql(sql: &str) -> String {
    let mut stripped = String::with_capacity(sql.len());
    for line in sql.lines() {
        let code = match line.find("--") {
            Some(idx) => &line[..idx],
            None => line,
        };
        stripped.push_str(code);
        stripped.push('\n');
    }
    stripped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// SQLx derives a migration's checksum from its SQL text alone, so we can
/// recompute the checksum of any SQL string (raw or normalized) the same way.
fn sqlx_checksum(sql: &str) -> Vec<u8> {
    Migration::new(
        0,
        Cow::Borrowed("checksum"),
        MigrationType::Simple,
        Cow::Owned(sql.to_string()),
        false,
    )
    .checksum
    .into_owned()
}

fn normalized_checksum(sql: &str) -> Vec<u8> {
    sqlx_checksum(&normalize_sql(sql))
}

fn semantic_change_error(backend: &str, version: i64) -> DbError {
    DbError::MigrationFailed(format!(
        "{backend} database migration failed: migration {version} was already applied, but its SQL \
         has changed in a way that is not just formatting. Never change the meaning of an applied \
         migration; add a new ordered migration instead. Restore migration {version} to its \
         released form, or reset the database from a backup."
    ))
}

const NORM_TABLE_SQLITE: &str = "CREATE TABLE IF NOT EXISTS _xrat_migration_norms \
     (version INTEGER PRIMARY KEY, norm_checksum BLOB NOT NULL)";

const NORM_TABLE_POSTGRES: &str = "CREATE TABLE IF NOT EXISTS _xrat_migration_norms \
     (version BIGINT PRIMARY KEY, norm_checksum BYTEA NOT NULL)";

/// Reconcile stored migration checksums with the current migration files so that
/// reformatting an already-applied migration does not break the database. The
/// normalized checksum of each applied migration is recorded in
/// `_xrat_migration_norms`; a raw mismatch is healed only when the normalized
/// SQL is unchanged (a formatting edit). A normalized mismatch means the
/// migration's meaning changed, which is rejected.
async fn repair_checksums_sqlite(pool: &SqlitePool) -> crate::db::Result<()> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    if exists.is_none() {
        return Ok(());
    }

    sqlx::query(NORM_TABLE_SQLITE).execute(pool).await?;

    for migration in SQLITE_MIGRATOR.iter() {
        if migration.migration_type.is_down_migration() {
            continue;
        }
        let version = migration.version;
        let file_raw = migration.checksum.to_vec();
        let file_norm = normalized_checksum(&migration.sql);

        let applied: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT checksum FROM _sqlx_migrations WHERE version = ? AND success = 1",
        )
        .bind(version)
        .fetch_optional(pool)
        .await?;
        let Some(applied) = applied else {
            continue;
        };

        if applied == file_raw {
            record_norm_sqlite(pool, version, &file_norm).await?;
            continue;
        }

        let stored_norm: Option<Vec<u8>> =
            sqlx::query_scalar("SELECT norm_checksum FROM _xrat_migration_norms WHERE version = ?")
                .bind(version)
                .fetch_optional(pool)
                .await?;

        if matches!(stored_norm, Some(ref norm) if *norm != file_norm) {
            return Err(semantic_change_error("SQLite", version));
        }

        sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
            .bind(file_raw.as_slice())
            .bind(version)
            .execute(pool)
            .await?;
        record_norm_sqlite(pool, version, &file_norm).await?;
    }
    Ok(())
}

async fn record_norm_sqlite(pool: &SqlitePool, version: i64, norm: &[u8]) -> crate::db::Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO _xrat_migration_norms (version, norm_checksum) VALUES (?, ?)",
    )
    .bind(version)
    .bind(norm)
    .execute(pool)
    .await?;
    Ok(())
}

async fn repair_checksums_postgres(pool: &PgPool) -> crate::db::Result<()> {
    let exists: Option<bool> = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = '_sqlx_migrations')",
    )
    .fetch_optional(pool)
    .await?;
    if exists != Some(true) {
        return Ok(());
    }

    sqlx::query(NORM_TABLE_POSTGRES).execute(pool).await?;

    for migration in POSTGRES_MIGRATOR.iter() {
        if migration.migration_type.is_down_migration() {
            continue;
        }
        let version = migration.version;
        let file_raw = migration.checksum.to_vec();
        let file_norm = normalized_checksum(&migration.sql);

        let applied: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT checksum FROM _sqlx_migrations WHERE version = $1 AND success = true",
        )
        .bind(version)
        .fetch_optional(pool)
        .await?;
        let Some(applied) = applied else {
            continue;
        };

        if applied == file_raw {
            record_norm_postgres(pool, version, &file_norm).await?;
            continue;
        }

        let stored_norm: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT norm_checksum FROM _xrat_migration_norms WHERE version = $1",
        )
        .bind(version)
        .fetch_optional(pool)
        .await?;

        if matches!(stored_norm, Some(ref norm) if *norm != file_norm) {
            return Err(semantic_change_error("PostgreSQL", version));
        }

        sqlx::query("UPDATE _sqlx_migrations SET checksum = $1 WHERE version = $2")
            .bind(file_raw.as_slice())
            .bind(version)
            .execute(pool)
            .await?;
        record_norm_postgres(pool, version, &file_norm).await?;
    }
    Ok(())
}

async fn record_norm_postgres(pool: &PgPool, version: i64, norm: &[u8]) -> crate::db::Result<()> {
    sqlx::query(
        "INSERT INTO _xrat_migration_norms (version, norm_checksum) VALUES ($1, $2) \
         ON CONFLICT (version) DO UPDATE SET norm_checksum = EXCLUDED.norm_checksum",
    )
    .bind(version)
    .bind(norm)
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

    /// A semantically identical but differently formatted version of a
    /// migration: comments stripped and whitespace collapsed. Its raw checksum
    /// differs from the file while its normalized checksum matches, simulating a
    /// reformat.
    fn reformatted_sql(version: i64) -> String {
        normalize_sql(&file_sql_sqlite(version))
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn file_raw_sqlite(version: i64) -> Vec<u8> {
        SQLITE_MIGRATOR
            .iter()
            .find(|migration| {
                migration.version == version && !migration.migration_type.is_down_migration()
            })
            .expect("migration version should exist")
            .checksum
            .to_vec()
    }

    fn file_sql_sqlite(version: i64) -> String {
        SQLITE_MIGRATOR
            .iter()
            .find(|migration| {
                migration.version == version && !migration.migration_type.is_down_migration()
            })
            .expect("migration version should exist")
            .sql
            .to_string()
    }

    /// Map each up migration's version to its hex-encoded *normalized* checksum.
    /// Normalized values are stable under cosmetic reformatting, so the manifest
    /// only changes when a migration's meaning changes or a migration is added.
    /// Down migrations are not tracked in `_sqlx_migrations`, so they are skipped.
    fn migration_checksums(migrator: &sqlx::migrate::Migrator) -> BTreeMap<String, String> {
        migrator
            .iter()
            .filter(|migration| !migration.migration_type.is_down_migration())
            .map(|migration| {
                (
                    migration.version.to_string(),
                    hex(&normalized_checksum(&migration.sql)),
                )
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

    async fn seed_migrations_table(pool: &SqlitePool) {
        sqlx::query(
            "CREATE TABLE _sqlx_migrations (version INTEGER PRIMARY KEY, description TEXT NOT NULL, installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, success BOOLEAN NOT NULL, checksum BLOB NOT NULL, execution_time BIGINT NOT NULL)",
        )
        .execute(pool)
        .await
        .expect("create migrations table");
    }

    async fn insert_applied(pool: &SqlitePool, version: i64, checksum: &[u8]) {
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)",
        )
        .bind(version)
        .bind(format!("migration {version}"))
        .bind(checksum)
        .execute(pool)
        .await
        .expect("seed applied migration");
    }

    async fn applied_checksum(pool: &SqlitePool, version: i64) -> Vec<u8> {
        sqlx::query_scalar("SELECT checksum FROM _sqlx_migrations WHERE version = ?")
            .bind(version)
            .fetch_one(pool)
            .await
            .expect("fetch applied checksum")
    }

    #[test]
    fn normalization_ignores_formatting() {
        let reformatted = reformatted_sql(19);
        assert_eq!(
            normalized_checksum(&reformatted),
            normalized_checksum(&file_sql_sqlite(19)),
        );
        assert_ne!(
            sqlx_checksum(&reformatted),
            file_raw_sqlite(19),
            "raw checksums should still differ; only the normalized form is stable",
        );
    }

    #[tokio::test]
    async fn heals_reformatted_migration_for_legacy_db_without_norm_record() {
        // The exact incident: migration 19 was applied with the old formatting,
        // then the file was reformatted. A database that predates the norm table
        // must still recover.
        let pool = memory_pool().await;
        seed_migrations_table(&pool).await;
        insert_applied(&pool, 19, &sqlx_checksum(&reformatted_sql(19))).await;

        repair_checksums_sqlite(&pool)
            .await
            .expect("formatting-only repair should succeed");

        assert_eq!(applied_checksum(&pool, 19).await, file_raw_sqlite(19));
    }

    #[tokio::test]
    async fn heals_formatting_change_when_normalized_checksum_matches() {
        let pool = memory_pool().await;
        seed_migrations_table(&pool).await;
        insert_applied(&pool, 19, &file_raw_sqlite(19)).await;
        // First pass records the normalized checksum for the in-sync migration.
        repair_checksums_sqlite(&pool).await.expect("record norm");

        // Now the migration appears reformatted: same meaning, different raw.
        sqlx::query("UPDATE _sqlx_migrations SET checksum = ? WHERE version = ?")
            .bind(sqlx_checksum(&reformatted_sql(19)))
            .bind(19_i64)
            .execute(&pool)
            .await
            .expect("simulate reformat");

        repair_checksums_sqlite(&pool)
            .await
            .expect("formatting heal should succeed");
        assert_eq!(applied_checksum(&pool, 19).await, file_raw_sqlite(19));
    }

    #[tokio::test]
    async fn rejects_semantic_change_to_applied_migration() {
        let pool = memory_pool().await;
        seed_migrations_table(&pool).await;
        let old_sql = "CREATE TABLE legacy_only (id INTEGER);";
        insert_applied(&pool, 19, &sqlx_checksum(old_sql)).await;
        sqlx::query(NORM_TABLE_SQLITE)
            .execute(&pool)
            .await
            .expect("create norm table");
        record_norm_sqlite(&pool, 19, &normalized_checksum(old_sql))
            .await
            .expect("record old norm");

        let err = repair_checksums_sqlite(&pool)
            .await
            .expect_err("semantic change must be rejected");
        assert!(format!("{err}").contains("not just formatting"));
    }

    /// Guard against changing the *meaning* of an applied/released migration.
    /// The manifest pins each migration's normalized checksum, so reformatting a
    /// migration does not trip this test; only a semantic edit or a new
    /// migration does, forcing a conscious manifest regeneration.
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
                        "{backend} migration {version} changed meaning (not just formatting)"
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
             Reformatting a migration is fine and will not trip this test. If you changed a \
             migration's meaning, never do that to an applied/released migration: add a new \
             ordered migration instead. If you added a new migration (or the change is \
             intentional and no database has applied it), regenerate the manifest with \
             `UPDATE_MIGRATION_MANIFEST=1 cargo test migration_files_match_committed_checksum_manifest`.",
            problems.join("\n  ")
        );
    }
}
