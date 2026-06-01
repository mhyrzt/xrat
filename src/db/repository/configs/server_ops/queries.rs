use sqlx::{QueryBuilder, Row, Sqlite};

use super::mapping::map_config_with_latest_test_row;
use super::select::{
    CONFIG_COLUMNS_ALIASED, LATEST_TEST_COLUMNS, LATEST_TEST_JOIN, push_api_filter,
};
use crate::db::connection::DbPool;
use crate::db::record::{ConfigListFilter, ConfigWithLatestTest};

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
