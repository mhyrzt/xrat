use super::*;

impl<'a> RuntimeService<'a> {
    pub(super) fn resolve_launch(
        &self,
        config: &ConfigRecord,
    ) -> crate::app::Result<ResolvedLaunch> {
        let runtime = &self.context.app_config.runtime;
        let socks = runtime.socks.enabled.then_some((
            runtime.socks.host.as_str(),
            runtime.socks.port,
            runtime.socks.udp,
        ));
        let http = runtime
            .http
            .enabled
            .then_some((runtime.http.host.as_str(), runtime.http.port));
        let shadowsocks = if runtime.shadowsocks.enabled {
            Some((
                runtime.shadowsocks.host.as_str(),
                runtime.shadowsocks.port,
                runtime.shadowsocks.method.as_str(),
                runtime.shadowsocks.password.resolve()?,
                runtime.shadowsocks.network.as_str(),
            ))
        } else {
            None
        };

        if socks.is_none() && http.is_none() && shadowsocks.is_none() {
            return Err(AppError::NoRuntimeInboundEnabled);
        }

        let node = node_from_record(config)?;
        let mut xray_config = generate_runtime_config_for_inbounds(&node, socks, http)
            .map_err(AppError::InvalidArgument)?;
        if let Some((host, port, method, password, network)) = &shadowsocks {
            xray_config.inbounds.push(Inbound {
                tag: "shadowsocks-in".to_string(),
                port: *port,
                listen: (*host).to_string(),
                protocol: "shadowsocks".to_string(),
                settings: Some(serde_json::json!({
                    "method": method,
                    "password": password,
                    "network": network
                })),
            });
        }

        let (ready_host, ready_port) = if let Some((host, port, _)) = socks {
            (connect_host_for_bind_host(host), port)
        } else if let Some((host, port)) = http {
            (connect_host_for_bind_host(host), port)
        } else if let Some((host, port, _, _, _)) = &shadowsocks {
            (connect_host_for_bind_host(host), *port)
        } else {
            unreachable!("validated at least one inbound")
        };
        let binary_path = match runtime.engine.as_str() {
            "xray" => self.context.runtime_paths.xray_path.clone(),
            "v2ray" => self.context.runtime_paths.v2ray_path.clone(),
            "sing-box" => self.context.runtime_paths.sing_box_path.clone(),
            other => PathBuf::from(other),
        };

        Ok(ResolvedLaunch {
            binary_path,
            config: xray_config,
            ready_host,
            ready_port,
            endpoints: RuntimeEndpoints {
                socks: socks.map(|(host, port, _)| RuntimeEndpoint {
                    host: host.to_string(),
                    port,
                }),
                http: http.map(|(host, port)| RuntimeEndpoint {
                    host: host.to_string(),
                    port,
                }),
                shadowsocks: shadowsocks.map(|(host, port, _, _, _)| RuntimeEndpoint {
                    host: host.to_string(),
                    port,
                }),
            },
        })
    }
}
