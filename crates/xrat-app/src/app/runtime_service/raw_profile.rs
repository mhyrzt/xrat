use super::*;

impl RuntimeService<'_> {
    pub(super) async fn resolve_raw_launch(
        &self,
        config: &ConfigRecord,
    ) -> crate::app::Result<ResolvedLaunch> {
        let runtime = &self.context.app_config.runtime;
        if runtime.engine != "xray" {
            return Err(AppError::InvalidArgument(
                "complete Xray profiles require runtime.engine = 'xray'".into(),
            ));
        }
        if runtime.tun.enabled || runtime.tun.kill_switch.is_enabled() {
            return Err(AppError::InvalidArgument("complete Xray profiles cannot use XRAT-managed TUN or kill switch; disable them before connecting".into()));
        }
        let document = crate::app::services::xray_profiles::load(self.context, config).await?;
        let value: serde_json::Value = serde_json::from_slice(document.as_bytes())?;
        let mut endpoints = RuntimeEndpoints::default();
        for inbound in value
            .get("inbounds")
            .and_then(serde_json::Value::as_array)
            .into_iter()
            .flatten()
        {
            let protocol = inbound
                .get("protocol")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            if protocol == "tun" {
                return Err(AppError::InvalidArgument("raw Xray TUN inbounds are unsupported because XRAT cannot verify their capture ownership".into()));
            }
            if !matches!(protocol, "socks" | "http") {
                continue;
            }
            let host = inbound
                .get("listen")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("0.0.0.0");
            let host = match host {
                "0.0.0.0" => "127.0.0.1",
                "::" => "::1",
                value => value,
            };
            if host != "localhost"
                && !host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|address| address.is_loopback())
            {
                continue;
            }
            let port = inbound
                .get("port")
                .and_then(|value| value.as_u64().or_else(|| value.as_str()?.parse().ok()))
                .and_then(|port| u16::try_from(port).ok())
                .filter(|port| *port != 0);
            let Some(port) = port else {
                continue;
            };
            let endpoint = RuntimeEndpoint {
                host: host.into(),
                port,
            };
            match protocol {
                "socks" if endpoints.socks.is_none() => endpoints.socks = Some(endpoint),
                "http" if endpoints.http.is_none() => endpoints.http = Some(endpoint),
                _ => {}
            }
        }
        let ready = endpoints.socks.as_ref().or(endpoints.http.as_ref()).ok_or_else(|| AppError::InvalidArgument("complete Xray profiles need a local TCP SOCKS or HTTP inbound with a single port for startup readiness; XRAT will not inject one".into()))?;
        Ok(ResolvedLaunch {
            binary_path: self.context.runtime_paths.xray_path.clone(),
            ready_host: ready.host.clone(),
            ready_port: ready.port,
            endpoints,
            validator: RuntimeValidator::Xray,
            config: RuntimeLaunchConfig::RawXray(document),
            allowed_endpoints: Vec::new(),
        })
    }
}
