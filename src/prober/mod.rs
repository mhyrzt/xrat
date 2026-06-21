pub mod download;
pub mod icmp;
pub mod real_delay;
pub mod tcp;
pub mod upload;

pub use download::{DownloadResult, download_speed_check};
pub use icmp::{IcmpResult, icmp_ping};
pub use real_delay::{RealDelayResult, real_delay_check};
pub use tcp::{TcpResult, tcp_check};
pub use upload::{UploadResult, upload_speed_check};

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
#[derive(Debug, Clone, Default)]
pub struct TestResult {
    pub icmp_ok: bool,
    pub icmp_ms: Option<u32>,
    pub tcp_ok: bool,
    pub tcp_ms: Option<u32>,
    pub real_delay_ok: bool,
    pub real_delay_ms: Option<u32>,
    pub download_ok: bool,
    pub download_mbps: Option<f64>,
    pub upload_ok: bool,
    pub upload_mbps: Option<f64>,
    pub ttfb_ms: Option<u32>,
    pub http_status: Option<u16>,
    pub dial_endpoint_ip: Option<String>,
    pub dial_endpoint_location: Option<String>,
    pub dial_endpoint_country: Option<String>,
    pub dial_endpoint_asn: Option<String>,
    pub dial_endpoint_geoip_source: Option<String>,
    pub dial_endpoint_fronting: Option<String>,
    pub failure_kind: Option<FailureKind>,
    pub failure_reason: Option<String>,
}
