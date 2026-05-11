use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::db::model::{ConnectionTestRecord, ConnectionTestRunRecord};

pub fn map_connection_test_row<R>(row: R) -> ConnectionTestRecord
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
    ConnectionTestRecord {
        id: row.get("id"),
        run_id: row.get("run_id"),
        config_id: row.get("config_id"),
        icmp_ok: row.get::<Option<i64>, _>("icmp_ok").map(|value| value != 0),
        icmp_ms: row.get("icmp_ms"),
        tcp_ok: row.get::<Option<i64>, _>("tcp_ok").map(|value| value != 0),
        tcp_ms: row.get("tcp_ms"),
        real_delay_ok: row
            .get::<Option<i64>, _>("real_delay_ok")
            .map(|value| value != 0),
        real_delay_ms: row.get("real_delay_ms"),
        download_mbps: row.get("download_mbps"),
        upload_mbps: row.get("upload_mbps"),
        connect_ms: row.get("connect_ms"),
        ttfb_ms: row.get("ttfb_ms"),
        http_status: row.get("http_status"),
        endpoint_ip: row.get("endpoint_ip"),
        endpoint_location: row.get("endpoint_location"),
        endpoint_country: row.get("endpoint_country"),
        endpoint_asn: row.get("endpoint_asn"),
        failure_kind: row.get("failure_kind"),
        failure_reason: row.get("failure_reason"),
        tested_at: row.get("tested_at"),
    }
}

pub fn map_connection_test_run_row<R>(row: R) -> ConnectionTestRunRecord
where
    R: Row,
    for<'a> &'a str: ColumnIndex<R>,
    i64: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    String: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    R::Database: Database,
{
    ConnectionTestRunRecord {
        id: row.get("id"),
        kind: row.get("kind"),
        created_at: row.get("created_at"),
    }
}
