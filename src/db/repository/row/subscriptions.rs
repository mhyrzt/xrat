use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::db::model::SubscriptionRecord;

pub fn map_subscription_row<R>(row: R) -> SubscriptionRecord
where
    R: Row,
    for<'a> &'a str: ColumnIndex<R>,
    i64: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    String: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<String>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    R::Database: Database,
{
    SubscriptionRecord {
        id: row.get("id"),
        source_kind: row.get("source_kind"),
        source_url: row.get("source_url"),
        name: row.get("name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        config_count: row.get("config_count"),
    }
}
