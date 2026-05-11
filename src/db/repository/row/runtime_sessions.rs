use sqlx::{ColumnIndex, Database, Decode, Row, Type};

use crate::db::DbError;
use crate::db::model::{RuntimeSessionRecord, RuntimeSessionStatus};

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
        last_transition_origin: row.get("last_transition_origin"),
        cooldown_until: row.get("cooldown_until"),
        last_failed_at: row.get("last_failed_at"),
        last_failed_reason_code: row.get("last_failed_reason_code"),
        started_at: row.get("started_at"),
        stopped_at: row.get("stopped_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
