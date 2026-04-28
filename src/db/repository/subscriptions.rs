use sqlx::{Postgres, QueryBuilder, Sqlite};

use super::row::map_subscription_row;
use crate::db::connection::DbPool;
use crate::db::model::{ImportSource, SourceKind, SubscriptionRecord};

pub async fn insert(
    pool: &DbPool,
    source: &ImportSource,
) -> Result<i64, Box<dyn std::error::Error>> {
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

pub async fn get_count(pool: &DbPool) -> Result<i64, Box<dyn std::error::Error>> {
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

pub async fn list(pool: &DbPool) -> Result<Vec<SubscriptionRecord>, Box<dyn std::error::Error>> {
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
