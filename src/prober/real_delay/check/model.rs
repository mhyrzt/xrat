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

impl RealDelayResult {
    pub fn failure(kind: FailureKind, reason: String) -> Self {
        Self {
            success: false,
            latency_ms: None,
            ttfb_ms: None,
            http_status: None,
            dial_endpoint_ip: None,
            failure_kind: Some(kind),
            failure_reason: Some(reason),
        }
    }
}
