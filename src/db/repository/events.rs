use sqlx::{Postgres, QueryBuilder, Sqlite};

use super::row::map_event_row;
use crate::db::connection::DbPool;
use crate::db::record::{EventFilter, EventRecord, NewEvent};

pub async fn insert(pool: &DbPool, event: &NewEvent) -> crate::db::Result<i64> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(
                "INSERT INTO events (level, source, kind, config_id, session_id, message, detail) VALUES (",
            );
            push_insert_values(&mut builder, event);
            builder.push(") RETURNING id");
            Ok(builder.build_query_scalar::<i64>().fetch_one(pool).await?)
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(
                "INSERT INTO events (level, source, kind, config_id, session_id, message, detail) VALUES (",
            );
            push_insert_values(&mut builder, event);
            builder.push(") RETURNING id");
            Ok(builder.build_query_scalar::<i64>().fetch_one(pool).await?)
        }
    }
}

/// Most recent events first, limited by `filter.limit`.
pub async fn query(pool: &DbPool, filter: &EventFilter) -> crate::db::Result<Vec<EventRecord>> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(BASE_SELECT);
            push_filters(&mut builder, None, filter);
            builder.push(" ORDER BY id DESC LIMIT ");
            builder.push_bind(filter.limit.max(0));
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_event_row)
                .collect())
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(BASE_SELECT);
            push_filters(&mut builder, None, filter);
            builder.push(" ORDER BY id DESC LIMIT ");
            builder.push_bind(filter.limit.max(0));
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_event_row)
                .collect())
        }
    }
}

/// Events with `id > after_id`, oldest first, for follow-mode cursoring.
pub async fn query_after(
    pool: &DbPool,
    after_id: i64,
    filter: &EventFilter,
) -> crate::db::Result<Vec<EventRecord>> {
    match pool {
        DbPool::Sqlite(pool) => {
            let mut builder = QueryBuilder::<Sqlite>::new(BASE_SELECT);
            push_filters(&mut builder, Some(after_id), filter);
            builder.push(" ORDER BY id ASC");
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_event_row)
                .collect())
        }
        DbPool::Postgres(pool) => {
            let mut builder = QueryBuilder::<Postgres>::new(BASE_SELECT);
            push_filters(&mut builder, Some(after_id), filter);
            builder.push(" ORDER BY id ASC");
            Ok(builder
                .build()
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(map_event_row)
                .collect())
        }
    }
}

const BASE_SELECT: &str = "SELECT id, level, source, kind, config_id, session_id, message, detail, created_at FROM events";

fn push_insert_values<'args, DB>(builder: &mut QueryBuilder<'args, DB>, event: &'args NewEvent)
where
    DB: sqlx::Database,
    Option<i64>: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    Option<&'args str>: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    builder
        .push_bind(event.level.as_str())
        .push(", ")
        .push_bind(event.source.as_str())
        .push(", ")
        .push_bind(event.kind.as_str())
        .push(", ")
        .push_bind(event.config_id)
        .push(", ")
        .push_bind(event.session_id)
        .push(", ")
        .push_bind(event.message.as_str())
        .push(", ")
        .push_bind(event.detail.as_deref());
}

fn push_filters<'args, DB>(
    builder: &mut QueryBuilder<'args, DB>,
    after_id: Option<i64>,
    filter: &'args EventFilter,
) where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    &'args str: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    let mut first = true;
    let mut separator = |builder: &mut QueryBuilder<'args, DB>| {
        builder.push(if first { " WHERE " } else { " AND " });
        first = false;
    };

    if let Some(after_id) = after_id {
        separator(builder);
        builder.push("id > ").push_bind(after_id);
    }

    if let Some(source) = filter.source.as_deref() {
        separator(builder);
        builder.push("source = ").push_bind(source);
    }

    if let Some(levels) = filter.levels.as_ref()
        && !levels.is_empty()
    {
        separator(builder);
        builder.push("level IN (");
        let mut separated = builder.separated(", ");
        for level in levels {
            separated.push_bind(level.as_str());
        }
        separated.push_unseparated(")");
    }
}
