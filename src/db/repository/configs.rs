use sqlx::{QueryBuilder, Row, SqlitePool};

use crate::db::model::{ConfigListFilter, ConfigRecord, ImportSummary};
use crate::model::Node;

pub async fn import_nodes(
    pool: &SqlitePool,
    subscription_id: i64,
    nodes: &[Node],
) -> Result<ImportSummary, Box<dyn std::error::Error>> {
    if !nodes.is_empty() {
        let mut builder = QueryBuilder::new(
            "INSERT INTO configs (subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config) ",
        );

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
                imported_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            "#,
        );

        builder.build().execute(pool).await?;
    }

    let total_configs = get_count(pool).await?;

    Ok(ImportSummary {
        subscription_id,
        imported_configs: nodes.len(),
        total_configs,
    })
}

pub async fn get_count(pool: &SqlitePool) -> Result<i64, Box<dyn std::error::Error>> {
    Ok(sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM configs")
        .fetch_one(pool)
        .await?)
}

pub async fn list(
    pool: &SqlitePool,
    filter: &ConfigListFilter,
) -> Result<Vec<ConfigRecord>, Box<dyn std::error::Error>> {
    let mut builder = QueryBuilder::new(
        "SELECT id, subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config, is_active, is_enabled, is_selected, imported_at, created_at, updated_at FROM configs WHERE 1 = 1",
    );

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

    let rows = builder.build().fetch_all(pool).await?;
    let configs = rows.into_iter().map(map_config_row).collect();
    Ok(configs)
}

pub async fn get_by_id(
    pool: &SqlitePool,
    id: i64,
) -> Result<Option<ConfigRecord>, Box<dyn std::error::Error>> {
    let row = sqlx::query(
        "SELECT id, subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config, is_active, is_enabled, is_selected, imported_at, created_at, updated_at FROM configs WHERE id = ?1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(map_config_row))
}

pub async fn get_selected(
    pool: &SqlitePool,
) -> Result<Option<ConfigRecord>, Box<dyn std::error::Error>> {
    let row = sqlx::query(
        "SELECT id, subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config, is_active, is_enabled, is_selected, imported_at, created_at, updated_at FROM configs WHERE is_selected = 1 ORDER BY updated_at DESC, id DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(map_config_row))
}

pub async fn get_active(
    pool: &SqlitePool,
) -> Result<Option<ConfigRecord>, Box<dyn std::error::Error>> {
    let row = sqlx::query(
        "SELECT id, subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config, is_active, is_enabled, is_selected, imported_at, created_at, updated_at FROM configs WHERE is_active = 1 ORDER BY updated_at DESC, id DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(map_config_row))
}

pub async fn get_flags(
    pool: &SqlitePool,
    dedup_key: &str,
) -> Result<(bool, bool, bool), Box<dyn std::error::Error>> {
    let row =
        sqlx::query("SELECT is_active, is_enabled, is_selected FROM configs WHERE dedup_key = ?1")
            .bind(dedup_key)
            .fetch_one(pool)
            .await?;

    Ok((
        row.get::<i64, _>(0) != 0,
        row.get::<i64, _>(1) != 0,
        row.get::<i64, _>(2) != 0,
    ))
}

pub async fn clear_all_selected(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("UPDATE configs SET is_selected = 0, updated_at = CURRENT_TIMESTAMP")
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn mark_selected(pool: &SqlitePool, id: i64) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        "UPDATE configs
         SET is_selected = 1, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1",
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn clear_all_active(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        "UPDATE configs
         SET is_active = 0, updated_at = CURRENT_TIMESTAMP
         WHERE is_active = 1",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_active(pool: &SqlitePool, id: i64) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        "UPDATE configs
         SET is_active = 1, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1",
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_enabled(
    pool: &SqlitePool,
    id: i64,
    enabled: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let enabled_flag = if enabled { 1 } else { 0 };
    sqlx::query(
        "UPDATE configs
         SET is_enabled = ?2,
             is_selected = CASE WHEN ?2 = 0 THEN 0 ELSE is_selected END,
             is_active = CASE WHEN ?2 = 0 THEN 0 ELSE is_active END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1",
    )
    .bind(id)
    .bind(enabled_flag)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query("DELETE FROM configs WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

fn map_config_row(row: sqlx::sqlite::SqliteRow) -> ConfigRecord {
    ConfigRecord {
        id: row.get("id"),
        subscription_id: row.get("subscription_id"),
        dedup_key: row.get("dedup_key"),
        protocol: row.get("protocol"),
        address: row.get("address"),
        port: row.get("port"),
        username: row.get("username"),
        uuid: row.get("uuid"),
        password: row.get("password"),
        method: row.get("method"),
        network: row.get("network"),
        tls: row.get("tls"),
        sni: row.get("sni"),
        host: row.get("host"),
        path: row.get("path"),
        name: row.get("name"),
        raw_config: row.get("raw_config"),
        is_active: row.get::<i64, _>("is_active") != 0,
        is_enabled: row.get::<i64, _>("is_enabled") != 0,
        is_selected: row.get::<i64, _>("is_selected") != 0,
        imported_at: row.get("imported_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
