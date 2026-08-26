use sqlx::{Postgres, QueryBuilder, Sqlite};

use super::row::map_subscription_row;
use crate::db::connection::DbPool;
use crate::db::record::{
    ImportSource, RefMatch, RefreshableSubscription, SourceKind, SubscriptionRecord,
};
use crate::support::time::now_epoch_seconds;

/// Resolve a ref prefix to a single subscription id.
pub async fn resolve_ref_prefix(pool: &DbPool, prefix: &str) -> crate::db::Result<RefMatch> {
    let like = format!("{prefix}%");
    let ids: Vec<i64> = match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query_scalar(
                "SELECT id FROM subscriptions WHERE ref LIKE ?1 ORDER BY id ASC LIMIT 2",
            )
            .bind(&like)
            .fetch_all(pool)
            .await?
        }
        DbPool::Postgres(pool) => {
            sqlx::query_scalar(
                "SELECT id FROM subscriptions WHERE ref LIKE $1 ORDER BY id ASC LIMIT 2",
            )
            .bind(&like)
            .fetch_all(pool)
            .await?
        }
    };
    Ok(match ids.len() {
        0 => RefMatch::None,
        1 => RefMatch::Unique(ids[0]),
        _ => RefMatch::Ambiguous,
    })
}

pub async fn insert(pool: &DbPool, source: &ImportSource) -> crate::db::Result<i64> {
    let new_ref = crate::support::refs::generate_ref();
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(
                "INSERT INTO subscriptions (ref, source_url, source_kind, name) VALUES (",
            );
            push_insert_values(&mut builder, &new_ref, source);
            builder.push(") RETURNING id");
            Ok(builder.build_query_scalar::<i64>().fetch_one(pool).await?)
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(
                "INSERT INTO subscriptions (ref, source_url, source_kind, name) VALUES (",
            );
            push_insert_values(&mut builder, &new_ref, source);
            builder.push(") RETURNING id");
            Ok(builder.build_query_scalar::<i64>().fetch_one(pool).await?)
        }
    }
}

fn push_insert_values<'args, DB>(
    builder: &mut QueryBuilder<'args, DB>,
    new_ref: &'args str,
    source: &'args ImportSource,
) where
    DB: sqlx::Database,
    Option<&'args str>: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    let source_url = matches!(source.kind, SourceKind::Url).then_some(source.value.as_str());
    builder
        .push_bind(new_ref)
        .push(", ")
        .push_bind(source_url)
        .push(", ")
        .push_bind(source.kind.as_str())
        .push(", ")
        .push_bind(source.name.as_deref());
}

pub async fn delete_with_configs(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    const SQLITE_CONFIG_IDS: &str = "SELECT id FROM configs WHERE subscription_id = ?1";
    const POSTGRES_CONFIG_IDS: &str = "SELECT id FROM configs WHERE subscription_id = $1";
    match pool {
        DbPool::Sqlite(pool) => {
            let mut tx = pool.begin().await?;
            sqlx::query(&format!(
                "DELETE FROM connection_tests WHERE config_id IN ({SQLITE_CONFIG_IDS})"
            ))
            .bind(id)
            .execute(&mut *tx)
            .await?;
            sqlx::query(&format!(
                "DELETE FROM runtime_sessions WHERE config_id IN ({SQLITE_CONFIG_IDS})"
            ))
            .bind(id)
            .execute(&mut *tx)
            .await?;
            sqlx::query("DELETE FROM configs WHERE subscription_id = ?1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM subscriptions WHERE id = ?1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
        }
        DbPool::Postgres(pool) => {
            let mut tx = pool.begin().await?;
            sqlx::query(&format!(
                "DELETE FROM connection_tests WHERE config_id IN ({POSTGRES_CONFIG_IDS})"
            ))
            .bind(id)
            .execute(&mut *tx)
            .await?;
            sqlx::query(&format!(
                "DELETE FROM runtime_sessions WHERE config_id IN ({POSTGRES_CONFIG_IDS})"
            ))
            .bind(id)
            .execute(&mut *tx)
            .await?;
            sqlx::query("DELETE FROM configs WHERE subscription_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            sqlx::query("DELETE FROM subscriptions WHERE id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
        }
    }
    Ok(())
}

pub async fn find_or_create(pool: &DbPool, source: &ImportSource) -> crate::db::Result<i64> {
    if matches!(source.kind, SourceKind::Url) && !source.value.is_empty() {
        let existing = match pool {
            DbPool::Sqlite(pool) => {
                sqlx::query_scalar::<_, i64>(
                    "SELECT id FROM subscriptions WHERE source_url = ? LIMIT 1",
                )
                .bind(&source.value)
                .fetch_optional(pool)
                .await?
            }
            DbPool::Postgres(pool) => {
                sqlx::query_scalar::<_, i64>(
                    "SELECT id FROM subscriptions WHERE source_url = $1 LIMIT 1",
                )
                .bind(&source.value)
                .fetch_optional(pool)
                .await?
            }
        };
        if let Some(id) = existing {
            return Ok(id);
        }
    }
    insert(pool, source).await
}

pub async fn set_name(pool: &DbPool, id: i64, name: &str) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query(
                "UPDATE subscriptions SET name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
            )
            .bind(name)
            .bind(id)
            .execute(pool)
            .await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query(
                "UPDATE subscriptions SET name = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2",
            )
            .bind(name)
            .bind(id)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

/// Stamp a subscription as refreshed now. `last_refreshed_at` holds epoch
/// seconds as text (matching the `cooldown_until` convention) so the scheduler
/// can compare it without timezone parsing.
pub async fn mark_refreshed(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    let now = now_epoch_seconds().to_string();
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE subscriptions SET last_refreshed_at = ? WHERE id = ?")
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE subscriptions SET last_refreshed_at = $1 WHERE id = $2")
                .bind(&now)
                .bind(id)
                .execute(pool)
                .await?;
        }
    }
    Ok(())
}

/// URL-backed subscriptions that are due for an automatic refresh: those never
/// refreshed, or last refreshed at or before `cutoff_epoch_secs` (now minus the
/// configured interval). Comparing on the persisted timestamp makes the
/// interval survive daemon restarts.
pub async fn list_refreshable_due(
    pool: &DbPool,
    cutoff_epoch_secs: i64,
) -> crate::db::Result<Vec<RefreshableSubscription>> {
    const SQLITE_SQL: &str = r#"
        SELECT id, source_url
        FROM subscriptions
        WHERE source_kind = 'url'
          AND source_url IS NOT NULL
          AND source_url <> ''
          AND (
              last_refreshed_at IS NULL
              OR CAST(last_refreshed_at AS INTEGER) <= ?1
          )
        ORDER BY id ASC
        "#;
    const POSTGRES_SQL: &str = r#"
        SELECT id, source_url
        FROM subscriptions
        WHERE source_kind = 'url'
          AND source_url IS NOT NULL
          AND source_url <> ''
          AND (
              last_refreshed_at IS NULL
              OR CAST(last_refreshed_at AS BIGINT) <= $1
          )
        ORDER BY id ASC
        "#;

    let map = |row: (i64, Option<String>)| {
        row.1.map(|source_url| RefreshableSubscription {
            id: row.0,
            source_url,
        })
    };

    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query_as::<_, (i64, Option<String>)>(SQLITE_SQL)
            .bind(cutoff_epoch_secs)
            .fetch_all(pool)
            .await?
            .into_iter()
            .filter_map(map)
            .collect()),
        DbPool::Postgres(pool) => Ok(sqlx::query_as::<_, (i64, Option<String>)>(POSTGRES_SQL)
            .bind(cutoff_epoch_secs)
            .fetch_all(pool)
            .await?
            .into_iter()
            .filter_map(map)
            .collect()),
    }
}

pub async fn get_count(pool: &DbPool) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM subscriptions",
        )
        .fetch_one(pool)
        .await?),
        DbPool::Postgres(pool) => Ok(sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM subscriptions",
        )
        .fetch_one(pool)
        .await?),
    }
}

pub async fn get_by_id(pool: &DbPool, id: i64) -> crate::db::Result<Option<SubscriptionRecord>> {
    const SQLITE_SQL: &str = r#"
        SELECT
            subscriptions.id,
            subscriptions.ref,
            subscriptions.source_kind,
            subscriptions.source_url,
            subscriptions.name,
            subscriptions.created_at,
            subscriptions.updated_at,
            COUNT(configs.id) AS config_count
        FROM subscriptions
        LEFT JOIN configs
            ON configs.subscription_id = subscriptions.id
            AND configs.is_deleted = 0
        WHERE subscriptions.id = ?1
        GROUP BY
            subscriptions.id,
            subscriptions.ref,
            subscriptions.source_kind,
            subscriptions.source_url,
            subscriptions.name,
            subscriptions.created_at,
            subscriptions.updated_at
        "#;
    const POSTGRES_SQL: &str = r#"
        SELECT
            subscriptions.id,
            subscriptions.ref,
            subscriptions.source_kind,
            subscriptions.source_url,
            subscriptions.name,
            subscriptions.created_at,
            subscriptions.updated_at,
            COUNT(configs.id) AS config_count
        FROM subscriptions
        LEFT JOIN configs
            ON configs.subscription_id = subscriptions.id
            AND configs.is_deleted = 0
        WHERE subscriptions.id = $1
        GROUP BY
            subscriptions.id,
            subscriptions.ref,
            subscriptions.source_kind,
            subscriptions.source_url,
            subscriptions.name,
            subscriptions.created_at,
            subscriptions.updated_at
        "#;

    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(SQLITE_SQL)
            .bind(id)
            .fetch_optional(pool)
            .await?
            .map(map_subscription_row)),
        DbPool::Postgres(pool) => Ok(sqlx::query(POSTGRES_SQL)
            .bind(id)
            .fetch_optional(pool)
            .await?
            .map(map_subscription_row)),
    }
}

pub async fn list(pool: &DbPool) -> crate::db::Result<Vec<SubscriptionRecord>> {
    const SQL: &str = r#"
        SELECT
            subscriptions.id,
            subscriptions.ref,
            subscriptions.source_kind,
            subscriptions.source_url,
            subscriptions.name,
            subscriptions.created_at,
            subscriptions.updated_at,
            COUNT(configs.id) AS config_count
        FROM subscriptions
        LEFT JOIN configs
            ON configs.subscription_id = subscriptions.id
            AND configs.is_deleted = 0
        GROUP BY
            subscriptions.id,
            subscriptions.ref,
            subscriptions.source_kind,
            subscriptions.source_url,
            subscriptions.name,
            subscriptions.created_at,
            subscriptions.updated_at
        ORDER BY subscriptions.id ASC
        "#;

    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(SQL)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(map_subscription_row)
            .collect()),
        DbPool::Postgres(pool) => Ok(sqlx::query(SQL)
            .fetch_all(pool)
            .await?
            .into_iter()
            .map(map_subscription_row)
            .collect()),
    }
}
