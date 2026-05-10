use sqlx::{Postgres, QueryBuilder, Sqlite};

use crate::db::connection::DbPool;
use crate::db::model::{ConfigListFilter, ConfigRecord, ImportSummary};
use crate::db::repository::row::map_config_row;
use crate::model::Node;

pub(super) const CONFIG_COLUMNS: &str = "id, subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config, is_active, is_enabled, is_selected, imported_at, created_at, updated_at";

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
