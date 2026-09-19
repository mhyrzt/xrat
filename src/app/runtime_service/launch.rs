use super::*;

impl<'a> RuntimeService<'a> {
    pub(super) fn resolve_launch(
        &self,
        config: &ConfigRecord,
    ) -> crate::app::Result<ResolvedLaunch> {
        let runtime = &self.context.app_config.runtime;
        // When listen_interface is set, all managed inbounds bind to that
        // interface's address instead of their configured host.
        let listen_addr = resolve_listen_interface_addr(runtime)?;
        let socks_host = listen_addr
            .clone()
            .unwrap_or_else(|| runtime.socks.host.clone());
        let http_host = listen_addr
            .clone()
            .unwrap_or_else(|| runtime.http.host.clone());
        let shadowsocks_host = listen_addr
            .clone()
            .unwrap_or_else(|| runtime.shadowsocks.host.clone());

        let socks = runtime.socks.enabled.then_some((
            socks_host.as_str(),
            runtime.socks.port,
            runtime.socks.udp,
        ));
        let http = runtime
            .http
            .enabled
            .then_some((http_host.as_str(), runtime.http.port));
        let shadowsocks = if runtime.shadowsocks.enabled {
            Some((
                shadowsocks_host.as_str(),
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

        let binary_path = match runtime.engine.as_str() {
            "xray" => self.context.runtime_paths.xray_path.clone(),
            "v2ray" => self.context.runtime_paths.v2ray_path.clone(),
            other => PathBuf::from(other),
        };
        let mut gen_options = build_xray_gen_options(runtime);
        gen_options.compatibility = crate::app::runtime_tuning::detect_xray_compatibility(
            runtime.xray_compatibility,
            &binary_path,
        );
        apply_xray_dns_options(&mut gen_options, &self.context.app_config.dns)?;
        apply_xray_routing_options(&mut gen_options, &self.context.app_config.routing);
        if gen_options.bind_address.is_some() {
            tracing::warn!(
                "[runtime.network].bind_address is set but the Xray engine cannot bind a source address; ignoring it"
            );
        }
        let mut xray_config =
            generate_runtime_config_for_inbounds_with_options(&node, socks, http, &gen_options)
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

        if runtime.stats.enabled {
            enable_stats_api(&mut xray_config, &runtime.stats.host, runtime.stats.port);
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
            validator: if runtime.engine == "v2ray" {
                RuntimeValidator::V2ray
            } else {
                RuntimeValidator::Xray
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
            if !udp {
                return Err(AppError::InvalidArgument(
                    "[runtime.socks].udp = false cannot be represented by sing-box 1.13; use Xray/V2Ray or enable UDP"
                        .to_string(),
                ));
            }
            inbounds.push(SingboxInbound::socks(
                "socks-in",
                host,
                port,
                self.singbox_socks_users()?,
            ));
        }
        if let Some((host, port)) = http {
            inbounds.push(SingboxInbound::http("http-in", host, port));
        }
        if let Some((host, port, method, password, network)) = &shadowsocks {
            inbounds.push(
                SingboxInbound::shadowsocks(
                    "shadowsocks-in",
                    *host,
                    *port,
                    *network,
                    *method,
                    password.clone(),
                )
                .map_err(AppError::InvalidArgument)?,
            );
        }

        let stats = &self.context.app_config.runtime.stats;
        let clash_api = if stats.enabled {
            if !is_loopback_listener(&stats.host) {
                return Err(AppError::InvalidArgument(format!(
                    "[runtime.stats].host = \"{}\" would expose the sing-box Clash API beyond loopback; use 127.0.0.1, ::1, or localhost",
                    stats.host
                )));
            }
            for (label, port) in [
                ("[runtime.socks].port", socks.map(|(_, port, _)| port)),
                ("[runtime.http].port", http.map(|(_, port)| port)),
                (
                    "[runtime.shadowsocks].port",
                    shadowsocks.as_ref().map(|(_, port, _, _, _)| *port),
                ),
            ] {
                if port == Some(stats.port) {
                    return Err(AppError::InvalidArgument(format!(
                        "[runtime.stats].port {} collides with {label} for the sing-box Clash API",
                        stats.port
                    )));
                }
            }
            Some(SingboxClashApi {
                external_controller: format!("{}:{}", stats.host, stats.port),
                secret: None,
            })
        } else {
            None
        };
        let routing = build_singbox_routing_options(&self.context.app_config.routing);
        let dns = build_singbox_dns_options(&self.context.app_config.dns)?;
        let mut config = generate_singbox_runtime_config_with_dns(
            node,
            inbounds,
            clash_api,
            Some(&routing),
            dns.as_ref(),
        )
        .map_err(AppError::InvalidArgument)?;
        if config.has_rule_sets() {
            let cache_path = self
                .context
                .runtime_paths
                .runtime_dir
                .join("singbox-cache.db");
            config.enable_cache_file(cache_path.display().to_string());
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
            validator: RuntimeValidator::Singbox,
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

fn is_loopback_listener(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host
            .trim_matches(|character| character == '[' || character == ']')
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

fn resolve_runtime_engine(
    configured_engine: &str,
    node: &crate::model::Node,
) -> crate::app::Result<RuntimeEngine> {
    match configured_engine {
        "xray" => Ok(RuntimeEngine::Xray),
        "v2ray" if matches!(node.protocol, Protocol::Hy2) => Err(AppError::InvalidArgument(
            "Hysteria2 requires Xray or sing-box; V2Ray does not support it".to_string(),
        )),
        "v2ray" => Ok(RuntimeEngine::Xray),
        "sing-box" => Ok(RuntimeEngine::Singbox),
        other => Err(AppError::InvalidArgument(format!(
            "unsupported runtime engine \"{other}\""
        ))),
    }
}
