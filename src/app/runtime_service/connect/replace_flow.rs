use super::*;
use std::net::TcpListener;

impl<'a> RuntimeService<'a> {
    pub async fn replace(&self, request: ReplaceRequest) -> crate::app::Result<ReplaceResult> {
        let active = match self.active_session_state().await? {
            ActiveSessionState::Running(session) => session,
            ActiveSessionState::Stale(_) | ActiveSessionState::None => {
                return Err(AppError::InvalidArgument(
                    "no running runtime session to replace".to_string(),
                ));
            }
        };
        let next_config_id = request
            .candidate_id
            .unwrap_or(active.config_id.ok_or_else(|| {
                AppError::InvalidArgument("active runtime session has no config id".to_string())
            })?);
        self.context
            .db
            .update_runtime_session_transition_metadata(
                active.id,
                None,
                None,
                Some("replace_started"),
                Some(&format!(
                    "trigger={:?}, candidate_id={}",
                    request.trigger, next_config_id
                )),
                Some("daemon"),
            )
            .await?;

        let staged = self.stage_replacement_runtime(next_config_id).await;
        let (next_config_id, session_id, new_pid) = match staged {
            Ok(value) => value,
            Err(err) => {
                self.context
                    .db
                    .update_runtime_session_transition_metadata(
                        active.id,
                        None,
                        None,
                        Some("replace_validation_failed"),
                        Some(&err.to_string()),
                        Some("daemon"),
                    )
                    .await?;
                self.context
                    .db
                    .update_runtime_session_transition_metadata(
                        active.id,
                        None,
                        None,
                        Some("replace_rollback_keep_old"),
                        Some("replacement candidate rejected before handoff"),
                        Some("daemon"),
                    )
                    .await?;
                return Err(err);
            }
        };

        self.context.db.set_active_config(next_config_id).await?;
        stop_session(self.context, &active).await?;

        Ok(ReplaceResult {
            old_session_id: active.id,
            new_config_id: next_config_id,
            new_session_id: session_id,
            new_pid,
        })
    }

    async fn stage_replacement_runtime(
        &self,
        next_config_id: i64,
    ) -> crate::app::Result<(i64, i64, u32)> {
        let Some(next_config) = self.context.db.get_config_by_id(next_config_id).await? else {
            return Err(AppError::InvalidArgument(format!(
                "config {} was not found",
                next_config_id
            )));
        };
        if !next_config.is_enabled {
            return Err(AppError::InvalidArgument(format!(
                "config {} is disabled",
                next_config_id
            )));
        }

        let mut launch = self.resolve_launch(&next_config)?;
        assign_ephemeral_inbound_ports(&mut launch)?;
        let session_id = self
            .context
            .db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(next_config.id),
                status: RuntimeSessionStatus::Starting,
                socks_host: launch
                    .endpoints
                    .socks
                    .as_ref()
                    .map(|inbound| inbound.host.clone()),
                socks_port: launch
                    .endpoints
                    .socks
                    .as_ref()
                    .map(|inbound| i64::from(inbound.port)),
                http_host: launch
                    .endpoints
                    .http
                    .as_ref()
                    .map(|inbound| inbound.host.clone()),
                http_port: launch
                    .endpoints
                    .http
                    .as_ref()
                    .map(|inbound| i64::from(inbound.port)),
                shadowsocks_host: launch
                    .endpoints
                    .shadowsocks
                    .as_ref()
                    .map(|inbound| inbound.host.clone()),
                shadowsocks_port: launch
                    .endpoints
                    .shadowsocks
                    .as_ref()
                    .map(|inbound| i64::from(inbound.port)),
                process_id: None,
                failure_reason: None,
                started_at: None,
                stopped_at: None,
            })
            .await?;

        let spawned = xray_runtime::spawn_detached(
            &launch.binary_path,
            &self.context.runtime_paths.runtime_dir,
            session_id,
            &launch.config,
            &launch.ready_host,
            launch.ready_port,
            Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
        )
        .await;
        let spawned = match spawned {
            Ok(process) => process,
            Err(err) => {
                self.context
                    .db
                    .update_runtime_session_state(
                        session_id,
                        RuntimeSessionStatus::Failed,
                        None,
                        None,
                        Some(&now_string()),
                        Some(&err.to_string()),
                    )
                    .await?;
                return Err(err);
            }
        };

        self.context
            .db
            .update_runtime_session_state(
                session_id,
                RuntimeSessionStatus::Running,
                Some(i64::from(spawned.pid)),
                Some(&now_string()),
                None,
                None,
            )
            .await?;
        Ok((next_config.id, session_id, spawned.pid))
    }
}

fn assign_ephemeral_inbound_ports(launch: &mut ResolvedLaunch) -> crate::app::Result<()> {
    if let Some(endpoint) = launch.endpoints.socks.as_mut() {
        endpoint.port = allocate_port(&endpoint.host)?;
    }
    if let Some(endpoint) = launch.endpoints.http.as_mut() {
        endpoint.port = allocate_port(&endpoint.host)?;
    }
    if let Some(endpoint) = launch.endpoints.shadowsocks.as_mut() {
        endpoint.port = allocate_port(&endpoint.host)?;
    }

    for inbound in &mut launch.config.inbounds {
        match inbound.tag.as_str() {
            "socks-in" => {
                if let Some(endpoint) = launch.endpoints.socks.as_ref() {
                    inbound.port = endpoint.port;
                }
            }
            "http-in" => {
                if let Some(endpoint) = launch.endpoints.http.as_ref() {
                    inbound.port = endpoint.port;
                }
            }
            "shadowsocks-in" => {
                if let Some(endpoint) = launch.endpoints.shadowsocks.as_ref() {
                    inbound.port = endpoint.port;
                }
            }
            _ => {}
        }
    }

    if let Some(endpoint) = launch.endpoints.socks.as_ref() {
        launch.ready_port = endpoint.port;
    } else if let Some(endpoint) = launch.endpoints.http.as_ref() {
        launch.ready_port = endpoint.port;
    } else if let Some(endpoint) = launch.endpoints.shadowsocks.as_ref() {
        launch.ready_port = endpoint.port;
    }

    Ok(())
}

fn allocate_port(host: &str) -> crate::app::Result<u16> {
    let bind_host = connect_host_for_bind_host(host);
    let listener = TcpListener::bind((bind_host.as_str(), 0)).map_err(|err| {
        AppError::InvalidArgument(format!("failed to allocate ephemeral inbound port: {err}"))
    })?;
    let port = listener
        .local_addr()
        .map_err(|err| {
            AppError::InvalidArgument(format!("failed to resolve ephemeral inbound port: {err}"))
        })?
        .port();
    drop(listener);
    Ok(port)
}
