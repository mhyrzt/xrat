use super::*;

#[derive(Clone, Debug, Serialize)]
pub(crate) struct TestOutputRow {
    pub(crate) id: i64,
    pub(crate) r#ref: String,
    pub(crate) name: Option<String>,
    pub(crate) protocol: String,
    pub(crate) address: String,
    pub(crate) port: i64,
    pub(crate) icmp_ms: Option<u32>,
    pub(crate) real_delay_ms: Option<u32>,
    pub(crate) download_mbps: Option<f64>,
    pub(crate) upload_mbps: Option<f64>,
    pub(crate) status: TestStatus,
    pub(crate) error: Option<String>,
    #[serde(skip_serializing)]
    pub(crate) tcp_ms: Option<u32>,
    #[serde(skip_serializing)]
    pub(crate) ttfb_ms: Option<u32>,
    #[serde(skip_serializing)]
    pub(crate) http_status: Option<u16>,
    #[serde(skip_serializing)]
    pub(crate) endpoint_ip: Option<String>,
    #[serde(skip_serializing)]
    pub(crate) endpoint_location: Option<String>,
    #[serde(skip_serializing)]
    pub(crate) endpoint_country: Option<String>,
    #[serde(skip_serializing)]
    pub(crate) endpoint_asn: Option<String>,
    #[serde(skip_serializing)]
    pub(crate) ran_icmp: bool,
    #[serde(skip_serializing)]
    pub(crate) ran_tcp: bool,
    #[serde(skip_serializing)]
    pub(crate) ran_real_delay: bool,
    #[serde(skip_serializing)]
    pub(crate) icmp_ok: bool,
    #[serde(skip_serializing)]
    pub(crate) tcp_ok: bool,
    #[serde(skip_serializing)]
    pub(crate) real_delay_ok: bool,
    #[serde(skip_serializing)]
    pub(crate) failure_kind: Option<String>,
    #[serde(skip_serializing)]
    pub(crate) elapsed_secs: f64,
}
