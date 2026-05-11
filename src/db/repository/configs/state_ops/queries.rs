use super::super::import_list::CONFIG_COLUMNS;
use crate::db::connection::DbPool;
use crate::db::model::ConfigRecord;
use crate::db::repository::row::map_config_row;

pub async fn get_selected(pool: &DbPool) -> crate::db::Result<Option<ConfigRecord>> {
    get_one_ordered(pool, "is_selected = 1").await
}

pub async fn get_active(pool: &DbPool) -> crate::db::Result<Option<ConfigRecord>> {
    get_one_ordered(pool, "is_active = 1").await
}

async fn get_one_ordered(
    pool: &DbPool,
    condition: &str,
) -> crate::db::Result<Option<ConfigRecord>> {
    let sql = format!(
        "SELECT {CONFIG_COLUMNS} FROM configs WHERE {condition} ORDER BY updated_at DESC, id DESC LIMIT 1"
    );
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(&sql)
            .fetch_optional(pool)
            .await?
            .map(map_config_row)),
        DbPool::Postgres(pool) => Ok(sqlx::query(&sql)
            .fetch_optional(pool)
            .await?
            .map(map_config_row)),
    }
}

pub async fn get_flags(pool: &DbPool, dedup_key: &str) -> crate::db::Result<(bool, bool, bool)> {
    match pool {
        DbPool::Sqlite(pool) => {
            let row: (i64, i64, i64) = sqlx::query_as(
                "SELECT is_active, is_enabled, is_selected FROM configs WHERE dedup_key = ?1",
            )
            .bind(dedup_key)
            .fetch_one(pool)
            .await?;
            Ok((row.0 != 0, row.1 != 0, row.2 != 0))
        }
        DbPool::Postgres(pool) => {
            let row: (i64, i64, i64) = sqlx::query_as(
                "SELECT is_active, is_enabled, is_selected FROM configs WHERE dedup_key = $1",
            )
            .bind(dedup_key)
            .fetch_one(pool)
            .await?;
            Ok((row.0 != 0, row.1 != 0, row.2 != 0))
        }
    }
}
