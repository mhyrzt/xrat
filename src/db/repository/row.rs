use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::db::DbError;
use crate::db::model::{
    CfScanResultRecord, ConfigRecord, ConnectionTestRecord, ConnectionTestRunRecord,
    RuntimeSessionRecord, RuntimeSessionStatus, SubscriptionRecord,
};

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

pub fn map_runtime_session_row<R>(row: R) -> crate::db::Result<RuntimeSessionRecord>
where
    R: Row,
    for<'a> &'a str: ColumnIndex<R>,
    i64: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<i64>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    String: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    Option<String>: for<'r> Decode<'r, R::Database> + Type<R::Database>,
    R::Database: Database,
{
    let status_value: String = row.get("status");
    let status = RuntimeSessionStatus::from_str(&status_value)
        .ok_or_else(|| DbError::InvalidRuntimeSessionStatus(status_value.clone()))?;

    Ok(RuntimeSessionRecord {
        id: row.get("id"),
        config_id: row.get("config_id"),
        status,
        socks_host: row.get("socks_host"),
        socks_port: row.get("socks_port"),
        http_host: row.get("http_host"),
        http_port: row.get("http_port"),
        shadowsocks_host: row.get("shadowsocks_host"),
        shadowsocks_port: row.get("shadowsocks_port"),
        process_id: row.get("process_id"),
        failure_reason: row.get("failure_reason"),
        owner_kind: row.get("owner_kind"),
        owner_instance_id: row.get("owner_instance_id"),
        last_transition_reason_code: row.get("last_transition_reason_code"),
        last_transition_reason_detail: row.get("last_transition_reason_detail"),
        started_at: row.get("started_at"),
        stopped_at: row.get("stopped_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

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
