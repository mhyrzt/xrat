use super::super::import_ops::CONFIG_COLUMNS;
use crate::db::connection::DbPool;
use crate::db::record::ConfigRecord;
use crate::db::repository::row::map_config_row;

pub async fn get_active(pool: &DbPool) -> crate::db::Result<Option<ConfigRecord>> {
    get_one_ordered(pool, "is_active = 1 AND is_deleted = 0").await
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
