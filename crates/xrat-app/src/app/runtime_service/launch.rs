use super::*;

impl<'a> RuntimeService<'a> {
    pub(super) fn resolve_launch(
        &self,
        config: &ConfigRecord,
    ) -> crate::app::Result<ResolvedLaunch> {
        let runtime = &self.context.app_config.runtime;
        crate::app::services::runtime_tuning::tun_validation::validate(
            &runtime.tun,
            &runtime.engine,
        )
        .map_err(AppError::InvalidArgument)?;
        crate::app::services::runtime_tuning::dns_validation::validate_tun_ranges(
            &self.context.app_config.dns,
            &runtime.tun,
        )
        .map_err(AppError::InvalidArgument)?;
        crate::app::services::runtime_tuning::dns_validation::validate(
            &self.context.app_config.dns,
            &runtime.engine,
            runtime.tun.enabled,
        )
        .map_err(AppError::InvalidArgument)?;
        crate::app::services::runtime_tuning::dns_validation::validate_listener_ports(
            &self.context.app_config.dns,
            runtime,
        )
        .map_err(AppError::InvalidArgument)?;
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
            if runtime.tun.enabled {
                return Err(AppError::InvalidArgument(
                    "[runtime.tun].enabled needs at least one enabled local inbound (for example [runtime.socks]) for readiness and non-TUN fallback"
                        .to_string(),
                ));
            }
            return Err(AppError::NoRuntimeInboundEnabled);
        }

        let mut node = node_from_record(config)?;
        let engine = resolve_runtime_engine(runtime.engine.as_str(), &node)?;
        if runtime.tun.enabled && runtime.engine == "v2ray" {
            return Err(AppError::InvalidArgument(
                "[runtime.tun].enabled is not supported by the V2Ray engine; use xray or sing-box"
                    .to_string(),
            ));
        }
        if engine == RuntimeEngine::Singbox {
            return self.resolve_singbox_launch(&node, socks, http, shadowsocks);
        }

        let mut resolved_hosts = Vec::new();
        if runtime.tun.enabled || !self.context.app_config.dns.resolvers.is_empty() {
            if node.address.parse::<std::net::IpAddr>().is_err() {
                if (node.tls.as_deref() == Some("tls") || node.tls.as_deref() == Some("reality"))
                    && node.sni.is_none()
                {
                    node.sni = Some(node.address.clone());
                }
                if matches!(node.network.as_str(), "ws" | "httpupgrade" | "xhttp")
                    && node.host.is_none()
                {
                    node.host = Some(node.address.clone());
                }
                let ip = self
                    .resolve_bootstrap_host(&node.address, node.port)?
                    .ok_or_else(|| {
                        AppError::InvalidArgument(format!(
                            "failed to resolve proxy endpoint \"{}\" for TUN capture",
                            node.address
                        ))
                    })?;
                resolved_hosts.push((node.address.clone(), ip.to_string()));
                node.address = ip.to_string();
            }
            for server in &self.context.app_config.dns.servers {
                if let Some(host) = extract_dns_server_host(server)?
                    && host.parse::<std::net::IpAddr>().is_err()
                    && !resolved_hosts.iter().any(|(h, _)| h == &host)
                {
                    let ip = self.resolve_bootstrap_host(&host, 443)?.ok_or_else(|| {
                        AppError::InvalidArgument(format!(
                            "failed to resolve DNS provider host \"{host}\" for TUN capture"
                        ))
                    })?;
                    resolved_hosts.push((host, ip.to_string()));
                }
            }
            for resolver in &self.context.app_config.dns.resolvers {
                let url = crate::app::services::runtime_tuning::dns_validation::endpoint(
                    &resolver.address,
                )
                .map_err(AppError::InvalidArgument)?;
                if let Some(host) = url.host_str()
                    && host
                        .trim_matches(['[', ']'])
                        .parse::<std::net::IpAddr>()
                        .is_err()
                {
                    let ip = self
                        .resolve_bootstrap_host(host, url.port().unwrap_or(53))?
                        .ok_or_else(|| {
                            AppError::InvalidArgument(
                                "[dns.resolvers] encrypted/hostname resolver bootstrap failed"
                                    .into(),
                            )
                        })?;
                    resolved_hosts.push((host.to_string(), ip.to_string()));
                }
            }
            if self.context.app_config.dns.fakeip.enabled {
                for host in &self.context.app_config.dns.fakeip.exclude {
                    if self.context.app_config.dns.hosts.contains_key(host)
                        || self
                            .context
                            .app_config
                            .dns
                            .hosts
                            .contains_key(&format!("full:{host}"))
                    {
                        continue;
                    }
                    let ip = self.resolve_bootstrap_host(host, 53)?.ok_or_else(|| {
                        AppError::InvalidArgument(
                            "[dns.fakeip].exclude hostname bootstrap failed".into(),
                        )
                    })?;
                    resolved_hosts.push((host.clone(), ip.to_string()));
                }
            }
        }

        let binary_path = match runtime.engine.as_str() {
            "xray" => self.context.runtime_paths.xray_path.clone(),
            "v2ray" => self.context.runtime_paths.v2ray_path.clone(),
            other => PathBuf::from(other),
        };
        if runtime.tun.enabled {
            ensure_xray_tun_supported_with_spawner(
                &binary_path,
                self.process_ports.spawner.clone(),
            )?;
        } else if (self.context.app_config.dns.outbound.is_some()
            || self.context.app_config.dns.fakeip.enabled)
            && crate::app::services::runtime_tuning::xray_binary_version_with_spawner(
                &binary_path,
                self.process_ports.spawner.clone(),
            )
            .is_none_or(|version| version < (26, 7, 11))
        {
            return Err(AppError::InvalidArgument(
                "[dns.outbound]/[dns.fakeip] requires a known Xray version >= 26.7.11; upgrade the configured core before enabling this DNS policy".into(),
            ));
        }
        if runtime.tun.enabled && !runtime.tun.route_exclude_address.is_empty() {
            return Err(AppError::InvalidArgument(
                    "[runtime.tun].route_exclude_address is not supported by the Xray engine; only sing-box can exclude destinations from TUN capture"
                        .to_string(),
                ));
        }
        let mut gen_options = build_xray_gen_options(runtime);
        gen_options.compatibility =
            crate::app::services::runtime_tuning::detect_xray_compatibility_with_spawner(
                runtime.xray_compatibility,
                &binary_path,
                self.process_ports.spawner.clone(),
            );
        apply_xray_dns_options(&mut gen_options, &self.context.app_config.dns)?;
        if let Some(dns) = &mut gen_options.dns {
            for (host, ip) in &resolved_hosts {
                dns.hosts.insert(
                    host.clone(),
                    xrat_engines::xray::XrayDnsHostValue::One(ip.clone()),
                );
            }
        }
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
                sniffing: None,
                tag: "shadowsocks-in".to_string(),
                port: Some(*port),
                listen: Some((*host).to_string()),
                protocol: "shadowsocks".to_string(),
                settings: Some(serde_json::json!({
                    "method": method,
                    "password": password,
                    "network": network
                })),
            });
        }

        if runtime.tun.enabled {
            let tun_options = XrayTunCaptureOptions {
                interface_name: &runtime.tun.interface_name,
                mtu: runtime.tun.mtu,
                address: &runtime.tun.address,
                auto_route: runtime.tun.auto_route,
                resolved_hosts: &resolved_hosts,
            };
            enable_tun_capture(&mut xray_config, &tun_options);
            let compiled_split =
                crate::app::services::split_tunnel::compile_split_rules(&runtime.tun);
            enable_tun_split_routing(&mut xray_config, &compiled_split.to_xray_options());
        }

        crate::app::services::runtime_tuning::apply_xray_dns_runtime(
            &mut xray_config,
            &self.context.app_config.dns,
            runtime.tun.enabled,
        )?;

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

    fn resolve_bootstrap_host(
        &self,
        host: &str,
        port: u16,
    ) -> crate::app::Result<Option<std::net::IpAddr>> {
        let dns = &self.context.app_config.dns;
        if dns.resolvers.is_empty() {
            return Ok(self.process_ports.resolver.resolve(host, port));
        }
        let resolver = dns
            .resolvers
            .iter()
            .find(|resolver| resolver.tag == dns.bootstrap_resolver)
            .ok_or_else(|| {
                AppError::InvalidArgument("[dns].bootstrap_resolver is missing".into())
            })?;
        let server = crate::app::services::runtime_tuning::dns_validation::bootstrap_socket(
            &resolver.address,
        )
        .map_err(AppError::InvalidArgument)?;
        let addresses = xrat_support::dns::resolve_udp(host, server, dns.query_strategy != "UseIPv6", dns.query_strategy != "UseIPv4", std::time::Duration::from_secs(2))
            .map_err(|_| AppError::InvalidArgument("[dns].bootstrap_resolver did not return a usable address; bootstrap failed before replacing any current connection".into()))?;
        Ok(addresses.into_iter().next())
    }

    fn resolve_singbox_launch(
        &self,
        node: &xrat_model::Node,
        socks: Option<(&str, u16, bool)>,
        http: Option<(&str, u16)>,
        shadowsocks: Option<(&str, u16, &str, String, &str)>,
    ) -> crate::app::Result<ResolvedLaunch> {
        let dns = &self.context.app_config.dns;
        if !dns.resolvers.is_empty() {
            if node.address.parse::<std::net::IpAddr>().is_err() {
                self.resolve_bootstrap_host(&node.address, node.port)?;
            }
            for resolver in &dns.resolvers {
                let endpoint = crate::app::services::runtime_tuning::dns_validation::endpoint(
                    &resolver.address,
                )
                .map_err(AppError::InvalidArgument)?;
                if let Some(host) = endpoint.host_str()
                    && host
                        .trim_matches(['[', ']'])
                        .parse::<std::net::IpAddr>()
                        .is_err()
                {
                    self.resolve_bootstrap_host(host, endpoint.port().unwrap_or(53))?;
                }
            }
        }
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

        let tun = &self.context.app_config.runtime.tun;
        let compiled_split = if tun.enabled {
            let compiled = crate::app::services::split_tunnel::compile_split_rules(tun);
            inbounds.push(
                SingboxInbound::tun(SingboxTunOptions {
                    tag: "tun-in".to_string(),
                    interface_name: tun.interface_name.clone(),
                    address: tun.address.clone(),
                    mtu: tun.mtu,
                    stack: tun.stack.clone(),
                    auto_route: tun.auto_route,
                    strict_route: tun.strict_route,
                    route_exclude_address: tun.route_exclude_address.clone(),
                })
                .map_err(AppError::InvalidArgument)?,
            );
            Some(compiled)
        } else {
            None
        };

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
        if let Some(compiled) = &compiled_split {
            config.enable_tun_route_with_split(&compiled.to_singbox_options());
        }
        crate::app::services::runtime_tuning::apply_singbox_dns_runtime(
            &mut config,
            &self.context.app_config.dns,
            tun.enabled,
            &self.context.runtime_paths.runtime_dir,
        )?;
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
    node: &xrat_model::Node,
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

pub(super) fn extract_dns_server_host(server: &str) -> crate::app::Result<Option<String>> {
    let server = server.trim();
    if matches!(server, "localhost" | "fakedns") || server.parse::<std::net::IpAddr>().is_ok() {
        return Ok(None);
    }
    let address = if server.contains("://") {
        server.to_string()
    } else {
        format!("udp://{server}")
    };
    let url = url::Url::parse(&address).map_err(|error| {
        AppError::InvalidArgument(format!("invalid DNS server \"{server}\": {error}"))
    })?;
    match url.host() {
        Some(url::Host::Domain(host)) if host.parse::<std::net::IpAddr>().is_err() => {
            Ok(Some(host.to_string()))
        }
        Some(_) => Ok(None),
        None => Err(AppError::InvalidArgument(format!(
            "DNS server \"{server}\" has no host"
        ))),
    }
}
