use crate::prober::FailureKind;

#[derive(Debug, Clone)]
pub struct RealDelayResult {
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub ttfb_ms: Option<u32>,
    pub http_status: Option<u16>,
    pub dial_endpoint_ip: Option<String>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}
