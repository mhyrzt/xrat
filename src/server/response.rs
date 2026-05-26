use serde::Serialize;

use crate::db::{ConfigRecord, ConnectionTestRecord};

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
    pub is_selected: bool,
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

pub fn latest_test_response(test: ConnectionTestRecord) -> ApiLatestTest {
    ApiLatestTest {
        tcp_ok: test.tcp_ok,
        tcp_ms: test.tcp_ms,
        real_delay_ok: test.real_delay_ok,
        real_delay_ms: test.real_delay_ms,
        download_mbps: test.download_mbps,
        upload_mbps: test.upload_mbps,
        connect_ms: test.connect_ms,
        ttfb_ms: test.ttfb_ms,
        http_status: test.http_status,
        failure_kind: test.failure_kind,
        failure_reason: test.failure_reason,
        tested_at: test.tested_at,
    }
}

pub fn summary_response(
    config: &ConfigRecord,
    latest_test: Option<&ConnectionTestRecord>,
) -> ApiConfigSummary {
    ApiConfigSummary {
        id: config.id,
        name: config.name.clone(),
        protocol: config.protocol.clone(),
        address: config.address.clone(),
        port: config.port,
        network: config.network.clone(),
        tls: config.tls.clone(),
        real_delay_ms: latest_test.and_then(|test| test.real_delay_ms),
        tcp_ok: latest_test.and_then(|test| test.tcp_ok),
        last_tested_at: latest_test.map(|test| test.tested_at.clone()),
    }
}

pub fn detail_response(
    config: ConfigRecord,
    latest_test: Option<ConnectionTestRecord>,
) -> ApiConfigDetail {
    ApiConfigDetail {
        id: config.id,
        subscription_id: config.subscription_id,
        dedup_key: config.dedup_key,
        protocol: config.protocol,
        address: config.address,
        port: config.port,
        name: config.name,
        network: config.network,
        tls: config.tls,
        sni: config.sni,
        host: config.host,
        path: config.path,
        is_active: config.is_active,
        is_enabled: config.is_enabled,
        is_selected: config.is_selected,
        imported_at: config.imported_at,
        created_at: config.created_at,
        updated_at: config.updated_at,
        latest_test: latest_test.map(latest_test_response),
    }
}
