use sqlx::{Postgres, QueryBuilder, Sqlite};

use super::row::map_subscription_row;
use crate::db::connection::DbPool;
use crate::db::record::{ImportSource, SourceKind, SubscriptionRecord};

pub async fn insert(pool: &DbPool, source: &ImportSource) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(
                "INSERT INTO subscriptions (source_url, source_kind, name) VALUES (",
            );
            push_insert_values(&mut builder, source);
            builder.push(") RETURNING id");
            Ok(builder.build_query_scalar::<i64>().fetch_one(pool).await?)
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(
                "INSERT INTO subscriptions (source_url, source_kind, name) VALUES (",
            );
            push_insert_values(&mut builder, source);
            builder.push(") RETURNING id");
            Ok(builder.build_query_scalar::<i64>().fetch_one(pool).await?)
        }
    }
}

fn push_insert_values<'args, DB>(builder: &mut QueryBuilder<'args, DB>, source: &'args ImportSource)
where
    DB: sqlx::Database,
    Option<&'args str>: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    let source_url = matches!(source.kind, SourceKind::Url).then_some(source.value.as_str());
    builder
        .push_bind(source_url)
        .push(", ")
        .push_bind(source.kind.as_str())
        .push(", ")
        .push_bind(source.name.as_deref());
}

pub async fn delete_with_configs(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("DELETE FROM configs WHERE subscription_id = ?")
                .bind(id)
                .execute(pool)
                .await?;
            sqlx::query("DELETE FROM subscriptions WHERE id = ?")
                .bind(id)
                .execute(pool)
                .await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("DELETE FROM configs WHERE subscription_id = $1")
                .bind(id)
                .execute(pool)
                .await?;
            sqlx::query("DELETE FROM subscriptions WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
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

pub async fn list(pool: &DbPool) -> crate::db::Result<Vec<SubscriptionRecord>> {
    const SQL: &str = r#"
        SELECT
            subscriptions.id,
            subscriptions.source_kind,
            subscriptions.source_url,
            subscriptions.name,
            subscriptions.created_at,
            subscriptions.updated_at,
            COUNT(configs.id) AS config_count
        FROM subscriptions
        LEFT JOIN configs ON configs.subscription_id = subscriptions.id
        GROUP BY
            subscriptions.id,
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
