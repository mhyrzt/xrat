use sqlx::SqlitePool;

use crate::db::models::{ImportSource, SourceKind};

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
