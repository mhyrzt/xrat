pub mod icmp;
pub mod real_delay;
pub mod tcp;

pub use icmp::{IcmpResult, icmp_ping};
pub use real_delay::{RealDelayResult, real_delay_check};
pub use tcp::{TcpResult, tcp_check};

/// Test failure classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureKind {
    Dns,
    Timeout,
    Refused,
    Unreachable,
    PermissionDenied,
    Tls,
    Auth,
    Process,
    Proxy,
    Unknown,
}

impl FailureKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dns => "dns",
            Self::Timeout => "timeout",
            Self::Refused => "refused",
            Self::Unreachable => "unreachable",
            Self::PermissionDenied => "permission_denied",
            Self::Tls => "tls",
            Self::Auth => "auth",
            Self::Process => "process",
            Self::Proxy => "proxy",
            Self::Unknown => "unknown",
        }
    }
}

/// Combined test result for a config
#[derive(Debug, Clone)]
pub struct TestResult {
    pub icmp_ok: bool,
    pub icmp_ms: Option<u32>,
    pub tcp_ok: bool,
    pub tcp_ms: Option<u32>,
    pub real_delay_ok: bool,
    pub real_delay_ms: Option<u32>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}

impl Default for TestResult {
    fn default() -> Self {
        Self {
            icmp_ok: false,
            icmp_ms: None,
            tcp_ok: false,
            tcp_ms: None,
            real_delay_ok: false,
            real_delay_ms: None,
            failure_kind: None,
            failure_reason: None,
        }
    }
}
