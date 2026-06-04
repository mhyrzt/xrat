use crate::db::connection::DbPool;
use crate::db::record::{EventFilter, EventRecord, NewEvent};
use crate::db::repository::events;

pub async fn record_event(pool: &DbPool, event: &NewEvent) -> crate::db::Result<i64> {
    events::insert(pool, event).await
}

pub async fn list_events(
    pool: &DbPool,
    filter: &EventFilter,
) -> crate::db::Result<Vec<EventRecord>> {
    events::query(pool, filter).await
}

pub async fn events_after(
    pool: &DbPool,
    after_id: i64,
    filter: &EventFilter,
) -> crate::db::Result<Vec<EventRecord>> {
    events::query_after(pool, after_id, filter).await
}
