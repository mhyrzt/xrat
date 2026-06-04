use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::db::record::{ConfigRecord, ConfigWithLatestTest};

pub(super) fn map_config_with_latest_test_row<R>(row: R) -> ConfigWithLatestTest
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
    let config = ConfigRecord {
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
        is_deleted: row.get::<i64, _>("is_deleted") != 0,
        deleted_at: row.get("deleted_at"),
        imported_at: row.get("imported_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };
    ConfigWithLatestTest {
        config,
        test_id: row.get("lt_id"),
        tcp_ok: row.get::<Option<i64>, _>("lt_tcp_ok").map(|v| v != 0),
        tcp_ms: row.get("lt_tcp_ms"),
        real_delay_ok: row
            .get::<Option<i64>, _>("lt_real_delay_ok")
            .map(|v| v != 0),
        real_delay_ms: row.get("lt_real_delay_ms"),
        download_mbps: row.get("lt_download_mbps"),
        upload_mbps: row.get("lt_upload_mbps"),
        connect_ms: row.get("lt_connect_ms"),
        ttfb_ms: row.get("lt_ttfb_ms"),
        http_status: row.get("lt_http_status"),
        failure_kind: row.get("lt_failure_kind"),
        failure_reason: row.get("lt_failure_reason"),
        tested_at: row.get("lt_tested_at"),
    }
}
