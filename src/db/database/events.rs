use super::Database;
use super::types::*;

impl Database {
    pub async fn record_event(&self, event: &NewEvent) -> crate::db::Result<i64> {
        repository::record_event(&self.pool, event).await
    }

    pub async fn list_events(&self, filter: &EventFilter) -> crate::db::Result<Vec<EventRecord>> {
        repository::list_events(&self.pool, filter).await
    }

    pub async fn events_after(
        &self,
        after_id: i64,
        filter: &EventFilter,
    ) -> crate::db::Result<Vec<EventRecord>> {
        repository::events_after(&self.pool, after_id, filter).await
    }
}
