use sqlx::{Postgres, QueryBuilder, Sqlite};

use super::row::map_config_row;
use crate::db::connection::DbPool;
use crate::db::model::{ConfigListFilter, ConfigRecord, ImportSummary};
use crate::model::Node;

const CONFIG_COLUMNS: &str = "id, subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config, is_active, is_enabled, is_selected, imported_at, created_at, updated_at";

pub async fn import_nodes(
    pool: &DbPool,
    subscription_id: i64,
    nodes: &[Node],
) -> crate::db::Result<ImportSummary> {
    if !nodes.is_empty() {
        match pool {
            DbPool::Sqlite(pool) => {
                let mut builder = QueryBuilder::<Sqlite>::new(
                    "INSERT INTO configs (subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config) ",
                );
                push_node_values(&mut builder, subscription_id, nodes);
                push_upsert_clause(&mut builder, "CURRENT_TIMESTAMP");
                builder.build().execute(pool).await?;
            }
            DbPool::Postgres(pool) => {
                let mut builder = QueryBuilder::<Postgres>::new(
                    "INSERT INTO configs (subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config) ",
                );
                push_node_values(&mut builder, subscription_id, nodes);
                push_upsert_clause(&mut builder, "CURRENT_TIMESTAMP::TEXT");
                builder.build().execute(pool).await?;
            }
        }
    }

    let total_configs = get_count(pool).await?;

    Ok(ImportSummary {
        subscription_id,
        imported_configs: nodes.len(),
        total_configs,
    })
}

fn push_node_values<'args, DB>(
    builder: &mut QueryBuilder<'args, DB>,
    subscription_id: i64,
    nodes: &'args [Node],
) where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args Option<String>: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    builder.push_values(nodes, |mut row, node| {
        row.push_bind(subscription_id)
            .push_bind(node.dedup_key_string())
            .push_bind(node.protocol.as_str())
            .push_bind(&node.address)
            .push_bind(i64::from(node.port))
            .push_bind(&node.username)
            .push_bind(&node.uuid)
            .push_bind(&node.password)
            .push_bind(&node.method)
            .push_bind(&node.network)
            .push_bind(&node.tls)
            .push_bind(&node.sni)
            .push_bind(&node.host)
            .push_bind(&node.path)
            .push_bind(&node.name)
            .push_bind(&node.raw_config);
    });
}

fn push_upsert_clause<DB>(builder: &mut QueryBuilder<'_, DB>, current_timestamp: &str)
where
    DB: sqlx::Database,
{
    builder.push(
        r#"
            ON CONFLICT(dedup_key) DO UPDATE SET
                subscription_id = excluded.subscription_id,
                protocol = excluded.protocol,
                address = excluded.address,
                port = excluded.port,
                username = excluded.username,
                uuid = excluded.uuid,
                password = excluded.password,
                method = excluded.method,
                network = excluded.network,
                tls = excluded.tls,
                sni = excluded.sni,
                host = excluded.host,
                path = excluded.path,
                name = excluded.name,
                raw_config = excluded.raw_config,
                imported_at = "#,
    );
    builder.push(current_timestamp);
    builder.push(
        r#",
                updated_at = "#,
    );
    builder.push(current_timestamp);
    builder.push(
        r#"
            "#,
    );
}

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

pub async fn clear_all_selected(pool: &DbPool) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(_) => {
            execute_no_bind(
                pool,
                "UPDATE configs SET is_selected = 0, updated_at = CURRENT_TIMESTAMP",
            )
            .await
        }
        DbPool::Postgres(_) => {
            execute_no_bind(
                pool,
                "UPDATE configs SET is_selected = 0, updated_at = CURRENT_TIMESTAMP::TEXT",
            )
            .await
        }
    }
}

pub async fn mark_selected(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    execute_id(
        pool,
        "UPDATE configs SET is_selected = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        "UPDATE configs SET is_selected = 1, updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1",
        id,
    )
    .await
}

pub async fn clear_all_active(pool: &DbPool) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(_) => {
            execute_no_bind(
                pool,
                "UPDATE configs SET is_active = 0, updated_at = CURRENT_TIMESTAMP WHERE is_active = 1",
            )
            .await
        }
        DbPool::Postgres(_) => {
            execute_no_bind(
                pool,
                "UPDATE configs SET is_active = 0, updated_at = CURRENT_TIMESTAMP::TEXT WHERE is_active = 1",
            )
            .await
        }
    }
}

pub async fn mark_active(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    execute_id(
        pool,
        "UPDATE configs SET is_active = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        "UPDATE configs SET is_active = 1, updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1",
        id,
    )
    .await
}

pub async fn set_enabled(pool: &DbPool, id: i64, enabled: bool) -> crate::db::Result<()> {
    let enabled_flag = if enabled { 1 } else { 0 };
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query("UPDATE configs SET is_enabled = ?2, is_selected = CASE WHEN ?2 = 0 THEN 0 ELSE is_selected END, is_active = CASE WHEN ?2 = 0 THEN 0 ELSE is_active END, updated_at = CURRENT_TIMESTAMP WHERE id = ?1")
                .bind(id).bind(enabled_flag).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query("UPDATE configs SET is_enabled = $2, is_selected = CASE WHEN $2 = 0 THEN 0 ELSE is_selected END, is_active = CASE WHEN $2 = 0 THEN 0 ELSE is_active END, updated_at = CURRENT_TIMESTAMP::TEXT WHERE id = $1")
                .bind(id).bind(enabled_flag).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn delete(pool: &DbPool, id: i64) -> crate::db::Result<()> {
    execute_id(
        pool,
        "DELETE FROM configs WHERE id = ?1",
        "DELETE FROM configs WHERE id = $1",
        id,
    )
    .await
}

async fn execute_no_bind(pool: &DbPool, sql: &str) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query(sql).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query(sql).execute(pool).await?;
        }
    }
    Ok(())
}

async fn execute_id(
    pool: &DbPool,
    sqlite_sql: &str,
    postgres_sql: &str,
    id: i64,
) -> crate::db::Result<()> {
    match pool {
        DbPool::Sqlite(pool) => {
            sqlx::query(sqlite_sql).bind(id).execute(pool).await?;
        }
        DbPool::Postgres(pool) => {
            sqlx::query(postgres_sql).bind(id).execute(pool).await?;
        }
    }
    Ok(())
}
