#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionTestInsert {
    pub run_id: Option<i64>,
    pub config_id: i64,
    pub icmp_ok: Option<bool>,
    pub icmp_ms: Option<i64>,
    pub tcp_ok: Option<bool>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ok: Option<bool>,
    pub real_delay_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub connect_ms: Option<i64>,
    pub ttfb_ms: Option<i64>,
    pub http_status: Option<i64>,
    pub endpoint_ip: Option<String>,
    pub endpoint_location: Option<String>,
    pub endpoint_country: Option<String>,
    pub endpoint_asn: Option<String>,
    pub failure_kind: Option<String>,
    pub failure_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionTestRecord {
    pub id: i64,
    pub run_id: Option<i64>,
    pub config_id: i64,
    pub icmp_ok: Option<bool>,
    pub icmp_ms: Option<i64>,
    pub tcp_ok: Option<bool>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ok: Option<bool>,
    pub real_delay_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub connect_ms: Option<i64>,
    pub ttfb_ms: Option<i64>,
    pub http_status: Option<i64>,
    pub endpoint_ip: Option<String>,
    pub endpoint_location: Option<String>,
    pub endpoint_country: Option<String>,
    pub endpoint_asn: Option<String>,
    pub failure_kind: Option<String>,
    pub failure_reason: Option<String>,
    pub tested_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionTestRunInsert {
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionTestRunRecord {
    pub id: i64,
    pub kind: String,
    pub created_at: String,
}
