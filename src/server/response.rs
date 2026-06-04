use serde::Serialize;

use crate::db::ConfigWithLatestTest;

#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ApiLatestTest {
    pub tcp_ok: Option<bool>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ok: Option<bool>,
    pub real_delay_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub connect_ms: Option<i64>,
    pub ttfb_ms: Option<i64>,
    pub http_status: Option<i64>,
    pub failure_kind: Option<String>,
    pub failure_reason: Option<String>,
    pub tested_at: String,
}

#[derive(Debug, Serialize)]
pub struct ApiConfigSummary {
    pub id: i64,
    pub name: Option<String>,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub network: String,
    pub tls: Option<String>,
    pub real_delay_ms: Option<i64>,
    pub tcp_ok: Option<bool>,
    pub last_tested_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiConfigDetail {
    pub id: i64,
    pub subscription_id: Option<i64>,
    pub dedup_key: String,
    pub protocol: String,
    pub address: String,
    pub port: i64,
    pub name: Option<String>,
    pub network: String,
    pub tls: Option<String>,
    pub sni: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub is_active: bool,
    pub is_enabled: bool,
    pub is_deleted: bool,
    pub deleted_at: Option<String>,
    pub imported_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub latest_test: Option<ApiLatestTest>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub total: usize,
    pub page: u64,
    pub per_page: u64,
    pub items: Vec<T>,
}

pub fn summary_from_joined(row: &ConfigWithLatestTest) -> ApiConfigSummary {
    ApiConfigSummary {
        id: row.config.id,
        name: row.config.name.clone(),
        protocol: row.config.protocol.clone(),
        address: row.config.address.clone(),
        port: row.config.port,
        network: row.config.network.clone(),
        tls: row.config.tls.clone(),
        real_delay_ms: row.real_delay_ms,
        tcp_ok: row.tcp_ok,
        last_tested_at: row.tested_at.clone(),
    }
}

pub fn detail_from_joined(row: ConfigWithLatestTest) -> ApiConfigDetail {
    let latest_test = row.test_id.map(|_| ApiLatestTest {
        tcp_ok: row.tcp_ok,
        tcp_ms: row.tcp_ms,
        real_delay_ok: row.real_delay_ok,
        real_delay_ms: row.real_delay_ms,
        download_mbps: row.download_mbps,
        upload_mbps: row.upload_mbps,
        connect_ms: row.connect_ms,
        ttfb_ms: row.ttfb_ms,
        http_status: row.http_status,
        failure_kind: row.failure_kind,
        failure_reason: row.failure_reason,
        tested_at: row.tested_at.unwrap_or_default(),
    });
    ApiConfigDetail {
        id: row.config.id,
        subscription_id: row.config.subscription_id,
        dedup_key: row.config.dedup_key,
        protocol: row.config.protocol,
        address: row.config.address,
        port: row.config.port,
        name: row.config.name,
        network: row.config.network,
        tls: row.config.tls,
        sni: row.config.sni,
        host: row.config.host,
        path: row.config.path,
        is_active: row.config.is_active,
        is_enabled: row.config.is_enabled,
        is_deleted: row.config.is_deleted,
        deleted_at: row.config.deleted_at,
        imported_at: row.config.imported_at,
        created_at: row.config.created_at,
        updated_at: row.config.updated_at,
        latest_test,
    }
}
