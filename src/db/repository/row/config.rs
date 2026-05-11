use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::db::model::ConfigRecord;

pub fn map_config_row<R>(row: R) -> ConfigRecord
where
    R: Row,
    for<'a> &'a str: ColumnIndex<R>,
    i64: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    String: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<String>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    R::Database: Database,
{
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
