use sqlx::{Postgres, QueryBuilder, Sqlite};

use crate::db::connection::DbPool;
use crate::db::record::ImportSummary;
use crate::model::Node;

use super::query::get_count;

pub async fn import_nodes(
    pool: &DbPool,
    subscription_id: i64,
    nodes: &[Node],
) -> crate::db::Result<ImportSummary> {
    let mut removed_configs = 0u64;
    if !nodes.is_empty() {
        // A stable ref per new config. On dedup_key conflict the upsert leaves
        // the existing ref untouched, so refs survive subscription refreshes.
        let refs: Vec<String> = nodes
            .iter()
            .map(|_| crate::support::refs::generate_ref())
            .collect();
        match pool {
            DbPool::Sqlite(pool) => {
                let mut builder = QueryBuilder::<Sqlite>::new(
                    "INSERT INTO configs (ref, subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config, extensions_json) ",
                );
                push_node_values(&mut builder, subscription_id, &refs, nodes);
                push_upsert_clause(&mut builder, "CURRENT_TIMESTAMP");
                builder.build().execute(pool).await?;
            }
            DbPool::Postgres(pool) => {
                let mut builder = QueryBuilder::<Postgres>::new(
                    "INSERT INTO configs (ref, subscription_id, dedup_key, protocol, address, port, username, uuid, password, method, network, tls, sni, host, path, name, raw_config, extensions_json) ",
                );
                push_node_values(&mut builder, subscription_id, &refs, nodes);
                push_upsert_clause(&mut builder, "CURRENT_TIMESTAMP::TEXT");
                builder.build().execute(pool).await?;
            }
        }

        removed_configs = reconcile_removed(pool, subscription_id, nodes).await?;
    }

    let total_configs = get_count(pool).await?;

    Ok(ImportSummary {
        subscription_id,
        imported_configs: nodes.len(),
        removed_configs,
        total_configs,
    })
}

/// Soft-delete configs attached to this subscription whose dedup keys are absent
/// from the freshly imported payload. This makes subscription refresh a
/// reconciliation: configs the provider dropped disappear from the active set
/// while staying recoverable, and a later provider re-add is undone by the
/// upsert clause (which clears `is_deleted`). Only called with a non-empty
/// payload, so a provider blip returning zero nodes cannot wipe a source.
async fn reconcile_removed(
    pool: &DbPool,
    subscription_id: i64,
    nodes: &[Node],
) -> crate::db::Result<u64> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(
                "UPDATE configs SET is_deleted = TRUE, deleted_at = CURRENT_TIMESTAMP WHERE is_deleted = 0 AND subscription_id = ",
            );
            push_reconcile_predicate(&mut builder, subscription_id, nodes);
            Ok(builder.build().execute(pool).await?.rows_affected())
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(
                "UPDATE configs SET is_deleted = TRUE, deleted_at = CURRENT_TIMESTAMP::TEXT WHERE is_deleted = FALSE AND subscription_id = ",
            );
            push_reconcile_predicate(&mut builder, subscription_id, nodes);
            Ok(builder.build().execute(pool).await?.rows_affected())
        }
    }
}

fn push_reconcile_predicate<'args, DB>(
    builder: &mut QueryBuilder<'args, DB>,
    subscription_id: i64,
    nodes: &'args [Node],
) where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    builder
        .push_bind(subscription_id)
        .push(" AND dedup_key NOT IN (");
    let mut separated = builder.separated(", ");
    for node in nodes {
        separated.push_bind(node.dedup_key_string());
    }
    separated.push_unseparated(")");
}

fn push_node_values<'args, DB>(
    builder: &mut QueryBuilder<'args, DB>,
    subscription_id: i64,
    refs: &'args [String],
    nodes: &'args [Node],
) where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args Option<String>: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    Option<String>: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    builder.push_values(refs.iter().zip(nodes), |mut row, (config_ref, node)| {
        row.push_bind(config_ref.as_str())
            .push_bind(subscription_id)
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
            .push_bind(&node.raw_config)
            .push_bind(
                node.extensions
                    .as_ref()
                    .and_then(|extensions| serde_json::to_string(extensions).ok()),
            );
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
                extensions_json = excluded.extensions_json,
                is_deleted = FALSE,
                deleted_at = NULL,
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
