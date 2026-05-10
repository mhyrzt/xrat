use super::*;

impl<'a> RuntimeService<'a> {
    pub fn new(context: &'a AppContext) -> Self {
        Self { context }
    }

    pub async fn connect(&self, request: ConnectRequest) -> crate::app::Result<ConnectResult> {
        let Some(config) = self.context.db.get_config_by_id(request.config_id).await? else {
            return Err(AppError::InvalidArgument(format!(
                "config {} was not found",
                request.config_id
            )));
        };
        if !config.is_enabled {
            return Err(AppError::InvalidArgument(format!(
                "config {} is disabled",
                request.config_id
            )));
        }

        match self.active_session_state().await? {
            ActiveSessionState::Running(session) => {
                if !self.context.app_config.runtime.replace_active_session {
                    tracing::warn!(
                        session_id = session.id,
                        "active runtime session blocks connect"
                    );
                    return Err(AppError::RuntimeSessionAlreadyActive);
                }

                self.disconnect().await?;
            }
            ActiveSessionState::Stale(session) => {
                tracing::warn!(
                    session_id = session.id,
                    "stale runtime session was reconciled before connect"
                );
            }
            ActiveSessionState::None => {}
        }

        let launch = self.resolve_launch(&config)?;
        let session_id = self
            .context
            .db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: Some(config.id),
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

        let process = match xray_runtime::spawn_detached(
            &launch.binary_path,
            &self.context.runtime_paths.runtime_dir,
            session_id,
            &launch.config,
            &launch.ready_host,
            launch.ready_port,
            Duration::from_millis(defaults::DEFAULT_XRAY_STARTUP_TIMEOUT_MS),
        )
        .await
        {
            Ok(process) => process,
            Err(error) => {
                self.context
                    .db
                    .update_runtime_session_state(
                        session_id,
                        RuntimeSessionStatus::Failed,
                        None,
                        None,
                        Some(&now_string()),
                        Some(&error.to_string()),
                    )
                    .await?;
                return Err(error);
            }
        };

        self.context
            .db
            .update_runtime_session_state(
                session_id,
                RuntimeSessionStatus::Running,
                Some(i64::from(process.pid)),
                Some(&now_string()),
                None,
                None,
            )
            .await?;
        self.context.db.set_active_config(config.id).await?;

        Ok(ConnectResult {
            config,
            session_id,
            pid: process.pid,
            runtime_config_path: process.paths.config_path,
            endpoints: launch.endpoints,
        })
    }

    pub async fn disconnect(&self) -> crate::app::Result<DisconnectResult> {
        let stopped_session = stop_active_session(self.context).await?;
        Ok(DisconnectResult { stopped_session })
    }

    pub async fn replace(&self, request: ReplaceRequest) -> crate::app::Result<ReplaceResult> {
        let active = match self.active_session_state().await? {
            ActiveSessionState::Running(session) => session,
            ActiveSessionState::Stale(_) | ActiveSessionState::None => {
                return Err(AppError::InvalidArgument(
                    "no running runtime session to replace".to_string(),
                ));
            }
        };
        let next_config_id = request.candidate_id.unwrap_or(active.config_id.ok_or_else(|| {
            AppError::InvalidArgument(
                "active runtime session has no config id".to_string(),
            )
        })?);
        let result = self
            .connect(ConnectRequest {
                config_id: next_config_id,
            })
            .await?;
        Ok(ReplaceResult {
            old_session_id: active.id,
            new_config_id: result.config.id,
            new_session_id: result.session_id,
            new_pid: result.pid,
        })
    }
}
