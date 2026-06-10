use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::db::record::GeoIpCacheRecord;

pub fn map_geoip_cache_row<R>(row: R) -> GeoIpCacheRecord
where
    R: Row,
    for<'a> &'a str: ColumnIndex<R>,
    i64: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    String: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<String>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    R::Database: Database,
{
    GeoIpCacheRecord {
        host: row.get("host"),
        ip: row.get("ip"),
        country: row.get("country"),
        location: row.get("location"),
        asn: row.get("asn"),
        resolved_at: row.get("resolved_at"),
    }
}
