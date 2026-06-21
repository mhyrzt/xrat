use super::*;

impl TestOutputRow {
    pub(crate) fn from_parts(parts: TestOutputParts<'_>) -> Self {
        let TestOutputParts {
            config,
            result,
            ran_icmp,
            ran_tcp,
            ran_real_delay,
            ran_download,
            ran_upload,
            elapsed,
        } = parts;
        let status = overall_status(
            result,
            ran_icmp,
            ran_tcp,
            ran_real_delay,
            ran_download,
            ran_upload,
        );

        Self {
            id: config.id,
            r#ref: config.r#ref.clone(),
            name: config.name.clone(),
            protocol: config.protocol.clone(),
            address: config.address.clone(),
            port: config.port,
            icmp_ms: result.icmp_ms,
            real_delay_ms: result.real_delay_ms,
            download_mbps: result.download_mbps,
            upload_mbps: result.upload_mbps,
            status,
            error: result.failure_reason.clone(),
            tcp_ms: result.tcp_ms,
            ttfb_ms: result.ttfb_ms,
            http_status: result.http_status,
            dial_endpoint_ip: result.dial_endpoint_ip.clone(),
            dial_endpoint_location: result.dial_endpoint_location.clone(),
            dial_endpoint_country: result.dial_endpoint_country.clone(),
            dial_endpoint_asn: result.dial_endpoint_asn.clone(),
            dial_endpoint_geoip_source: result.dial_endpoint_geoip_source.clone(),
            dial_endpoint_fronting: result.dial_endpoint_fronting.clone(),
            ran_icmp,
            ran_tcp,
            ran_real_delay,
            icmp_ok: result.icmp_ok,
            tcp_ok: result.tcp_ok,
            real_delay_ok: result.real_delay_ok,
            failure_kind: result
                .failure_kind
                .as_ref()
                .map(|kind| kind.as_str().to_string()),
            elapsed_secs: elapsed.as_secs_f64(),
        }
    }

    pub(crate) fn connection_test_insert(&self, run_id: Option<i64>) -> ConnectionTestInsert {
        ConnectionTestInsert {
            run_id,
            config_id: self.id,
            icmp_ok: self.ran_icmp.then_some(self.icmp_ok),
            icmp_ms: self.icmp_ms.map(i64::from),
            tcp_ok: self.ran_tcp.then_some(self.tcp_ok),
            tcp_ms: self.tcp_ms.map(i64::from),
            real_delay_ok: self.ran_real_delay.then_some(self.real_delay_ok),
            real_delay_ms: self.real_delay_ms.map(i64::from),
            download_mbps: self.download_mbps,
            upload_mbps: self.upload_mbps,
            connect_ms: self.tcp_ms.map(i64::from),
            ttfb_ms: self.ttfb_ms.map(i64::from),
            http_status: self.http_status.map(i64::from),
            dial_endpoint_ip: self.dial_endpoint_ip.clone(),
            dial_endpoint_location: self.dial_endpoint_location.clone(),
            dial_endpoint_country: self.dial_endpoint_country.clone(),
            dial_endpoint_asn: self.dial_endpoint_asn.clone(),
            dial_endpoint_geoip_source: self.dial_endpoint_geoip_source.clone(),
            dial_endpoint_fronting: self.dial_endpoint_fronting.clone(),
            failure_kind: self.failure_kind.clone(),
            failure_reason: self.error.clone(),
        }
    }
}

pub(crate) struct TestOutputParts<'a> {
    pub(crate) config: &'a ConfigRecord,
    pub(crate) result: &'a TestResult,
    pub(crate) ran_icmp: bool,
    pub(crate) ran_tcp: bool,
    pub(crate) ran_real_delay: bool,
    pub(crate) ran_download: bool,
    pub(crate) ran_upload: bool,
    pub(crate) elapsed: Duration,
}
