use std::time::Duration;

use crate::prober::FailureKind;

#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub success: bool,
    pub mbps: Option<f64>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

impl DownloadResult {
    pub(super) fn success(byte_count: u64, elapsed: Duration) -> Self {
        Self {
            success: true,
            mbps: Some(calculate_mbps(byte_count, elapsed)),
            failure_kind: None,
            failure_reason: None,
        }
    }

    pub(super) fn failure(kind: FailureKind, reason: String) -> Self {
        Self {
            success: false,
            mbps: None,
            failure_kind: Some(kind),
            failure_reason: Some(reason),
        }
    }

    pub(super) fn process_failure(reason: String) -> Self {
        Self::failure(FailureKind::Process, reason)
    }
}

pub(crate) fn calculate_mbps(byte_count: u64, elapsed: Duration) -> f64 {
    let seconds = elapsed.as_secs_f64().max(f64::EPSILON);
    (byte_count as f64 * 8.0) / seconds / 1_000_000.0
}
