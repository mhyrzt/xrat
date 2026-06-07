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
        let engine = resolve_runtime_engine(runtime.engine.as_str(), &node)?;
        if engine == RuntimeEngine::Singbox {
            return self.resolve_singbox_launch(&node, socks, http, shadowsocks);
        }

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
            other => PathBuf::from(other),
        };

        Ok(ResolvedLaunch {
            binary_path,
            config: RuntimeLaunchConfig::Xray(xray_config),
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

    fn resolve_singbox_launch(
        &self,
        node: &crate::model::Node,
        socks: Option<(&str, u16, bool)>,
        http: Option<(&str, u16)>,
        shadowsocks: Option<(&str, u16, &str, String, &str)>,
    ) -> crate::app::Result<ResolvedLaunch> {
        let mut inbounds = Vec::new();
        if let Some((host, port, udp)) = socks {
            inbounds.push(SingboxInbound {
                kind: "socks".to_string(),
                tag: "socks-in".to_string(),
                listen: host.to_string(),
                listen_port: port,
                network: udp.then_some("udp".to_string()),
                method: None,
                password: None,
                users: self.singbox_socks_users()?,
            });
        }
        if let Some((host, port)) = http {
            inbounds.push(SingboxInbound {
                kind: "http".to_string(),
                tag: "http-in".to_string(),
                listen: host.to_string(),
                listen_port: port,
                network: None,
                method: None,
                password: None,
                users: None,
            });
        }
        if let Some((host, port, method, password, network)) = &shadowsocks {
            inbounds.push(SingboxInbound {
                kind: "shadowsocks".to_string(),
                tag: "shadowsocks-in".to_string(),
                listen: (*host).to_string(),
                listen_port: *port,
                network: Some((*network).to_string()),
                method: Some((*method).to_string()),
                password: Some(password.clone()),
                users: None,
            });
        }

        let config = generate_singbox_runtime_config(node, inbounds, None)
            .map_err(AppError::InvalidArgument)?;
        let (ready_host, ready_port) = if let Some((host, port, _)) = socks {
            (connect_host_for_bind_host(host), port)
        } else if let Some((host, port)) = http {
            (connect_host_for_bind_host(host), port)
        } else if let Some((host, port, _, _, _)) = &shadowsocks {
            (connect_host_for_bind_host(host), *port)
        } else {
            unreachable!("validated at least one inbound")
        };

        Ok(ResolvedLaunch {
            binary_path: self.context.runtime_paths.sing_box_path.clone(),
            config: RuntimeLaunchConfig::Singbox(config),
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

    fn singbox_socks_users(&self) -> crate::app::Result<Option<Vec<SingboxInboundUser>>> {
        let auth = &self.context.app_config.runtime.socks.auth;
        if !auth.enabled {
            return Ok(None);
        }
        let Some(username) = &auth.username else {
            return Ok(None);
        };
        let Some(password) = &auth.password else {
            return Ok(None);
        };

        Ok(Some(vec![SingboxInboundUser {
            username: username.clone(),
            password: password.resolve()?,
        }]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimeEngine {
    Xray,
    Singbox,
}

fn resolve_runtime_engine(
    configured_engine: &str,
    node: &crate::model::Node,
) -> crate::app::Result<RuntimeEngine> {
    if matches!(node.protocol, Protocol::Hy2) {
        return Ok(RuntimeEngine::Singbox);
    }

    match configured_engine {
        "xray" | "v2ray" => Ok(RuntimeEngine::Xray),
        "sing-box" => Err(AppError::InvalidArgument(format!(
            "managed sing-box runtime currently supports hy2 configs only; protocol {} cannot be connected with [runtime].engine = \"sing-box\" yet",
            node.protocol
        ))),
        other => Err(AppError::InvalidArgument(format!(
            "unsupported runtime engine \"{other}\""
        ))),
    }
}
