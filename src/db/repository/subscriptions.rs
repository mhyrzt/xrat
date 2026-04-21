use sqlx::{Row, SqlitePool};

use crate::db::model::{ImportSource, SourceKind, SubscriptionRecord};

pub async fn insert(
    pool: &SqlitePool,
    source: &ImportSource,
) -> Result<i64, Box<dyn std::error::Error>> {
    let source_url = matches!(source.kind, SourceKind::Url).then_some(source.value.as_str());
    let result = sqlx::query(
        r#"
        INSERT INTO subscriptions (source_url, source_kind, name)
        VALUES (?1, ?2, ?3)
        "#,
    )
    .bind(source_url)
    .bind(source.kind.as_str())
    .bind(source.name.as_deref())
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

pub async fn get_count(pool: &SqlitePool) -> Result<i64, Box<dyn std::error::Error>> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM subscriptions")
            .fetch_one(pool)
            .await?,
    )
}

pub async fn list(
    pool: &SqlitePool,
) -> Result<Vec<SubscriptionRecord>, Box<dyn std::error::Error>> {
    let rows = sqlx::query(
        r#"
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
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| SubscriptionRecord {
            id: row.get("id"),
            source_kind: row.get("source_kind"),
            source_url: row.get("source_url"),
            name: row.get("name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            config_count: row.get("config_count"),
        })
        .collect())
}
