#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionTestInsert {
    pub config_id: i64,
    pub icmp_ok: Option<bool>,
    pub icmp_ms: Option<i64>,
    pub tcp_ok: Option<bool>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ok: Option<bool>,
    pub real_delay_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub failure_kind: Option<String>,
    pub failure_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConnectionTestRecord {
    pub id: i64,
    pub config_id: i64,
    pub icmp_ok: Option<bool>,
    pub icmp_ms: Option<i64>,
    pub tcp_ok: Option<bool>,
    pub tcp_ms: Option<i64>,
    pub real_delay_ok: Option<bool>,
    pub real_delay_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub failure_kind: Option<String>,
    pub failure_reason: Option<String>,
    pub tested_at: String,
}
