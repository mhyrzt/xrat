use sqlx::{Postgres, QueryBuilder, Sqlite};

use crate::db::connection::DbPool;
use crate::db::record::{ConfigListFilter, ConfigRecord};
use crate::db::repository::row::map_config_row;

pub const CONFIG_COLUMNS: &str = "id, subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config, is_active, is_enabled, is_selected, imported_at, created_at, updated_at";

pub async fn get_count(pool: &DbPool) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM configs")
            .fetch_one(pool)
            .await?),
        DbPool::Postgres(pool) => Ok(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM configs")
            .fetch_one(pool)
            .await?),
    }
}

pub async fn list(
    pool: &DbPool,
    filter: &ConfigListFilter,
) -> crate::db::Result<Vec<ConfigRecord>> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(format!(
                "SELECT {CONFIG_COLUMNS} FROM configs WHERE 1 = 1"
            ));
            push_filter(&mut builder, filter);
            let rows = builder.build().fetch_all(pool).await?;
            Ok(rows.into_iter().map(map_config_row).collect())
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(format!(
                "SELECT {CONFIG_COLUMNS} FROM configs WHERE 1 = 1"
            ));
            push_filter(&mut builder, filter);
            let rows = builder.build().fetch_all(pool).await?;
            Ok(rows.into_iter().map(map_config_row).collect())
        }
    }
}

fn push_filter<'args, DB>(builder: &mut QueryBuilder<'args, DB>, filter: &ConfigListFilter)
where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    if filter.only_enabled {
        builder.push(" AND is_enabled = 1");
    }
    if filter.only_selected {
        builder.push(" AND is_selected = 1");
    }
    if filter.only_active {
        builder.push(" AND is_active = 1");
    }
    if let Some(subscription_id) = filter.subscription_id {
        builder.push(" AND subscription_id = ");
        builder.push_bind(subscription_id);
    }
    if let Some(protocol) = &filter.protocol {
        builder.push(" AND protocol = ");
        builder.push_bind(protocol.clone());
    }
    builder.push(" ORDER BY id ASC");
}

pub async fn get_by_id(pool: &DbPool, id: i64) -> crate::db::Result<Option<ConfigRecord>> {
    match pool {
        DbPool::Sqlite(pool) => Ok(sqlx::query(&format!(
            "SELECT {CONFIG_COLUMNS} FROM configs WHERE id = ?1"
        ))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .map(map_config_row)),
        DbPool::Postgres(pool) => Ok(sqlx::query(&format!(
            "SELECT {CONFIG_COLUMNS} FROM configs WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .map(map_config_row)),
    }
}
