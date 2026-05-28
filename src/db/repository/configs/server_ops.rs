use sqlx::{ColumnIndex, Database, Decode, QueryBuilder, Row, Sqlite, Type};

use crate::db::connection::DbPool;
use crate::db::record::{ConfigListFilter, ConfigRecord, ConfigWithLatestTest};

const CONFIG_COLUMNS_ALIASED: &str = "c.id, c.subscription_id, c.dedup_key, c.protocol, c.address, c.port, c.username, c.uuid, c.password, c.method, c.network, c.tls, c.sni, c.host, c.path, c.name, c.raw_config, c.is_active, c.is_enabled, c.is_selected, c.is_deleted, c.deleted_at, c.imported_at, c.created_at, c.updated_at";

const LATEST_TEST_COLUMNS: &str = "ct.id AS lt_id, ct.tcp_ok AS lt_tcp_ok, ct.tcp_ms AS lt_tcp_ms, ct.real_delay_ok AS lt_real_delay_ok, ct.real_delay_ms AS lt_real_delay_ms, ct.download_mbps AS lt_download_mbps, ct.upload_mbps AS lt_upload_mbps, ct.connect_ms AS lt_connect_ms, ct.ttfb_ms AS lt_ttfb_ms, ct.http_status AS lt_http_status, ct.failure_kind AS lt_failure_kind, ct.failure_reason AS lt_failure_reason, ct.tested_at AS lt_tested_at";

const LATEST_TEST_JOIN: &str = "LEFT JOIN connection_tests ct ON ct.id = (SELECT ct2.id FROM connection_tests ct2 WHERE ct2.config_id = c.id ORDER BY ct2.tested_at DESC, ct2.id DESC LIMIT 1)";

fn push_api_filter<'args, DB>(builder: &mut QueryBuilder<'args, DB>, filter: &ConfigListFilter)
where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    if filter.only_deleted {
        builder.push(" AND c.is_deleted = 1");
    } else if !filter.include_deleted {
        builder.push(" AND c.is_deleted = 0");
    }
    if filter.only_enabled {
        builder.push(" AND c.is_enabled = 1");
    }
    if filter.only_selected {
        builder.push(" AND c.is_selected = 1");
    }
    if filter.only_active {
        builder.push(" AND c.is_active = 1");
    }
    if let Some(subscription_id) = filter.subscription_id {
        builder.push(" AND c.subscription_id = ");
        builder.push_bind(subscription_id);
    }
    if let Some(protocol) = &filter.protocol {
        builder.push(" AND c.protocol = ");
        builder.push_bind(protocol.clone());
    }
}

fn map_config_with_latest_test_row<R>(row: R) -> ConfigWithLatestTest
where
    R: Row,
    for<'a> &'a str: ColumnIndex<R>,
    i64: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<i64>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<f64>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    String: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<String>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    R::Database: Database,
{
    let config = ConfigRecord {
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
        is_deleted: row.get::<i64, _>("is_deleted") != 0,
        deleted_at: row.get("deleted_at"),
        imported_at: row.get("imported_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };
    ConfigWithLatestTest {
        config,
        test_id: row.get("lt_id"),
        tcp_ok: row.get::<Option<i64>, _>("lt_tcp_ok").map(|v| v != 0),
        tcp_ms: row.get("lt_tcp_ms"),
        real_delay_ok: row
            .get::<Option<i64>, _>("lt_real_delay_ok")
            .map(|v| v != 0),
        real_delay_ms: row.get("lt_real_delay_ms"),
        download_mbps: row.get("lt_download_mbps"),
        upload_mbps: row.get("lt_upload_mbps"),
        connect_ms: row.get("lt_connect_ms"),
        ttfb_ms: row.get("lt_ttfb_ms"),
        http_status: row.get("lt_http_status"),
        failure_kind: row.get("lt_failure_kind"),
        failure_reason: row.get("lt_failure_reason"),
        tested_at: row.get("lt_tested_at"),
    }
}

pub async fn list_with_latest_tests(
    pool: &DbPool,
    filter: &ConfigListFilter,
) -> crate::db::Result<Vec<ConfigWithLatestTest>> {
    let select = format!(
        "SELECT {CONFIG_COLUMNS_ALIASED}, {LATEST_TEST_COLUMNS} FROM configs c {LATEST_TEST_JOIN} WHERE 1 = 1"
    );
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(select);
            push_api_filter(&mut builder, filter);
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_config_with_latest_test_row)
                .collect())
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<sqlx::Postgres>::new(select);
            push_api_filter(&mut builder, filter);
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_config_with_latest_test_row)
                .collect())
        }
    }
}

pub async fn list_top_by_real_delay(
    pool: &DbPool,
    limit: i64,
    filter: &ConfigListFilter,
) -> crate::db::Result<Vec<ConfigWithLatestTest>> {
    let select = format!(
        "SELECT {CONFIG_COLUMNS_ALIASED}, {LATEST_TEST_COLUMNS} FROM configs c {LATEST_TEST_JOIN} WHERE 1 = 1"
    );
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(select);
            push_api_filter(&mut builder, filter);
            builder.push(" AND ct.real_delay_ms IS NOT NULL");
            builder.push(" ORDER BY ct.real_delay_ms ASC");
            builder.push(" LIMIT ");
            builder.push_bind(limit);
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_config_with_latest_test_row)
                .collect())
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<sqlx::Postgres>::new(select);
            push_api_filter(&mut builder, filter);
            builder.push(" AND ct.real_delay_ms IS NOT NULL");
            builder.push(" ORDER BY ct.real_delay_ms ASC");
            builder.push(" LIMIT ");
            builder.push_bind(limit);
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_config_with_latest_test_row)
                .collect())
        }
    }
}

pub async fn count_filtered(pool: &DbPool, filter: &ConfigListFilter) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder =
                QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM configs c WHERE 1 = 1");
            push_api_filter(&mut builder, filter);
            Ok(builder.build().fetch_one(pool).await?.get::<i64, _>(0))
        }
        DbPool::Postgres(pool) => {
            let mut builder =
                QueryBuilder::<sqlx::Postgres>::new("SELECT COUNT(*) FROM configs c WHERE 1 = 1");
            push_api_filter(&mut builder, filter);
            Ok(builder.build().fetch_one(pool).await?.get::<i64, _>(0))
        }
    }
}

pub async fn list_paginated_with_latest_tests(
    pool: &DbPool,
    filter: &ConfigListFilter,
    offset: i64,
    limit: i64,
) -> crate::db::Result<Vec<ConfigWithLatestTest>> {
    let select = format!(
        "SELECT {CONFIG_COLUMNS_ALIASED}, {LATEST_TEST_COLUMNS} FROM configs c {LATEST_TEST_JOIN} WHERE 1 = 1"
    );
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(select);
            push_api_filter(&mut builder, filter);
            builder.push(" LIMIT ");
            builder.push_bind(limit);
            builder.push(" OFFSET ");
            builder.push_bind(offset);
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_config_with_latest_test_row)
                .collect())
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<sqlx::Postgres>::new(select);
            push_api_filter(&mut builder, filter);
            builder.push(" LIMIT ");
            builder.push_bind(limit);
            builder.push(" OFFSET ");
            builder.push_bind(offset);
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_config_with_latest_test_row)
                .collect())
        }
    }
}

pub async fn get_with_latest_test(
    pool: &DbPool,
    id: i64,
) -> crate::db::Result<Option<ConfigWithLatestTest>> {
    let select = format!(
        "SELECT {CONFIG_COLUMNS_ALIASED}, {LATEST_TEST_COLUMNS} FROM configs c {LATEST_TEST_JOIN} WHERE c.id = "
    );
    match pool {
        DbPool::Sqlite(pool) => {
            let sql = format!("{select}?1");
            Ok(sqlx::query(&sql)
                .bind(id)
                .fetch_optional(pool)
                .await?
                .map(map_config_with_latest_test_row))
        }
        DbPool::Postgres(pool) => {
            let sql = format!("{select}$1");
            Ok(sqlx::query(&sql)
                .bind(id)
                .fetch_optional(pool)
                .await?
                .map(map_config_with_latest_test_row))
        }
    }
}
