use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::db::record::EventRecord;

pub fn map_event_row<R>(row: R) -> EventRecord
where
    R: Row,
    for<'a> &'a str: ColumnIndex<R>,
    i64: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<i64>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    String: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<String>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    R::Database: Database,
{
    EventRecord {
        id: row.get("id"),
        level: row.get("level"),
        source: row.get("source"),
        kind: row.get("kind"),
        config_id: row.get("config_id"),
        session_id: row.get("session_id"),
        message: row.get("message"),
        detail: row.get("detail"),
        created_at: row.get("created_at"),
    }
}
