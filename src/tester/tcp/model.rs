use super::FailureKind;

#[derive(Debug, Clone)]
pub struct TcpResult {
    pub success: bool,
    pub latency_ms: Option<u32>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}
