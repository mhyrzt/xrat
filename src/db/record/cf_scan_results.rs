#[derive(Clone, Debug, PartialEq)]
pub struct CfScanResultUpsert {
    pub ip: String,
    pub latency_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CfScanResultRecord {
    pub id: i64,
    pub ip: String,
    pub latency_ms: Option<i64>,
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub error: Option<String>,
    pub last_scanned_at: String,
}
