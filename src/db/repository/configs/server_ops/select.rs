use sqlx::QueryBuilder;

use crate::db::record::ConfigListFilter;

pub(super) const CONFIG_COLUMNS_ALIASED: &str = "c.id, c.subscription_id, c.dedup_key, c.protocol, c.address, c.port, c.username, c.uuid, c.password, c.method, c.network, c.tls, c.sni, c.host, c.path, c.name, c.raw_config, c.is_active, c.is_enabled, c.is_selected, c.is_deleted, c.deleted_at, c.imported_at, c.created_at, c.updated_at";

pub(super) const LATEST_TEST_COLUMNS: &str = "ct.id AS lt_id, ct.tcp_ok AS lt_tcp_ok, ct.tcp_ms AS lt_tcp_ms, ct.real_delay_ok AS lt_real_delay_ok, ct.real_delay_ms AS lt_real_delay_ms, ct.download_mbps AS lt_download_mbps, ct.upload_mbps AS lt_upload_mbps, ct.connect_ms AS lt_connect_ms, ct.ttfb_ms AS lt_ttfb_ms, ct.http_status AS lt_http_status, ct.failure_kind AS lt_failure_kind, ct.failure_reason AS lt_failure_reason, ct.tested_at AS lt_tested_at";

pub(super) const LATEST_TEST_JOIN: &str = "LEFT JOIN connection_tests ct ON ct.id = (SELECT ct2.id FROM connection_tests ct2 WHERE ct2.config_id = c.id ORDER BY ct2.tested_at DESC, ct2.id DESC LIMIT 1)";

pub(super) fn push_api_filter<'args, DB>(
    builder: &mut QueryBuilder<'args, DB>,
    filter: &ConfigListFilter,
) where
    DB: sqlx::Database,
    i64: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
    String: sqlx::Encode<'args, DB> + sqlx::Type<DB>,
{
    if filter.only_deleted {
        builder.push(" AND c.is_deleted = 1");
    } else if !filter.include_deleted {
        builder.push(" AND c.is_deleted = 0");
    }
    if filter.only_enabled {
        builder.push(" AND c.is_enabled = 1");
    }
    if filter.only_selected {
        builder.push(" AND c.is_selected = 1");
    }
    if filter.only_active {
        builder.push(" AND c.is_active = 1");
    }
    if let Some(subscription_id) = filter.subscription_id {
        builder.push(" AND c.subscription_id = ");
        builder.push_bind(subscription_id);
    }
    if let Some(protocol) = &filter.protocol {
        builder.push(" AND c.protocol = ");
        builder.push_bind(protocol.clone());
    }
}
