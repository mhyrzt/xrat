use super::*;

impl TestOutputRow {
    pub(super) fn from_parts(
        config: &ConfigRecord,
        result: &TestResult,
        ran_icmp: bool,
        ran_tcp: bool,
        ran_real_delay: bool,
        ran_download: bool,
        ran_upload: bool,
        elapsed: Duration,
    ) -> Self {
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
            endpoint_ip: result.endpoint_ip.clone(),
            endpoint_location: result.endpoint_location.clone(),
            endpoint_country: result.endpoint_country.clone(),
            endpoint_asn: result.endpoint_asn.clone(),
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

    pub(super) fn connection_test_insert(&self, run_id: Option<i64>) -> ConnectionTestInsert {
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
            endpoint_ip: self.endpoint_ip.clone(),
            endpoint_location: self.endpoint_location.clone(),
            endpoint_country: self.endpoint_country.clone(),
            endpoint_asn: self.endpoint_asn.clone(),
            failure_kind: self.failure_kind.clone(),
            failure_reason: self.error.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum TestStatus {
    Ok,
    Failed,
    Skipped,
}

impl TestStatus {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

pub(super) fn overall_status(
    result: &TestResult,
    ran_icmp: bool,
    ran_tcp: bool,
    ran_real_delay: bool,
    ran_download: bool,
    ran_upload: bool,
) -> TestStatus {
    if !ran_icmp && !ran_tcp && !ran_real_delay && !ran_download && !ran_upload {
        return TestStatus::Skipped;
    }

    let success = if ran_upload {
        result.upload_ok
    } else if ran_download {
        result.download_ok
    } else if ran_real_delay {
        result.real_delay_ok
    } else if ran_tcp {
        result.tcp_ok
    } else {
        result.icmp_ok
    };

    if success {
        TestStatus::Ok
    } else {
        TestStatus::Failed
    }
}

pub(super) fn node_from_record(config: &ConfigRecord) -> crate::app::Result<Node> {
    let protocol = match config.protocol.as_str() {
        "vless" => crate::model::Protocol::Vless,
        "vmess" => crate::model::Protocol::Vmess,
        "ss" => crate::model::Protocol::Ss,
        "trojan" => crate::model::Protocol::Trojan,
        "http" => crate::model::Protocol::Http,
        "socks5" => crate::model::Protocol::Socks5,
        other => return Err(AppError::UnsupportedProtocol(other.to_string())),
    };

    Ok(Node {
        protocol,
        address: config.address.clone(),
        port: config.port as u16,
        username: config.username.clone(),
        uuid: config.uuid.clone(),
        password: config.password.clone(),
        method: config.method.clone(),
        network: config.network.clone(),
        tls: config.tls.clone(),
        sni: config.sni.clone(),
        host: config.host.clone(),
        path: config.path.clone(),
        name: config.name.clone(),
        extensions: None,
        raw_config: config.raw_config.clone(),
    })
}
