use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::db::record::CfScanResultRecord;

pub fn map_cf_scan_result_row<R>(row: R) -> CfScanResultRecord
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
    CfScanResultRecord {
        id: row.get("id"),
        ip: row.get("ip"),
        latency_ms: row.get("latency_ms"),
        download_mbps: row.get("download_mbps"),
        upload_mbps: row.get("upload_mbps"),
        error: row.get("error"),
        last_scanned_at: row.get("last_scanned_at"),
    }
}
