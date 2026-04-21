use sqlx::{QueryBuilder, Row, SqlitePool};

use crate::db::models::ImportSummary;
use crate::model::Node;

pub async fn import_nodes(
    pool: &SqlitePool,
    subscription_id: i64,
    nodes: &[Node],
) -> Result<ImportSummary, Box<dyn std::error::Error>> {
    if !nodes.is_empty() {
        let mut builder = QueryBuilder::new(
            "INSERT INTO configs (subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name) ",
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
                .push_bind(&node.name);
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
                imported_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP,
                is_deleted = 0,
                deleted_at = NULL
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

pub async fn get_flags(
    pool: &SqlitePool,
    dedup_key: &str,
) -> Result<(bool, bool, bool, bool), Box<dyn std::error::Error>> {
    let row = sqlx::query(
        "SELECT is_active, is_enabled, is_deleted, is_selected FROM configs WHERE dedup_key = ?1",
    )
    .bind(dedup_key)
    .fetch_one(pool)
    .await?;

    Ok((
        row.get::<i64, _>(0) != 0,
        row.get::<i64, _>(1) != 0,
        row.get::<i64, _>(2) != 0,
        row.get::<i64, _>(3) != 0,
    ))
}

pub async fn mark_deleted(
    pool: &SqlitePool,
    dedup_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        "UPDATE configs SET is_deleted = 1, deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE dedup_key = ?1",
    )
    .bind(dedup_key)
    .execute(pool)
    .await?;
    Ok(())
}
