use super::*;

impl<'a> RuntimeService<'a> {
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
        if config.is_deleted {
            return Err(AppError::InvalidArgument(format!(
                "config {} is deleted",
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

        let process =
            match spawn_runtime(&launch, &self.context.runtime_paths.runtime_dir, session_id).await
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
        self.context
            .db
            .update_runtime_session_transition_metadata(
                session_id,
                Some("cli"),
                None,
                Some("manual_connect"),
                Some("runtime connect request succeeded"),
                Some("cli"),
            )
            .await?;
        self.context.db.set_active_config(config.id).await?;

        Ok(ConnectResult {
            config,
            session_id,
            pid: process.pid,
            runtime_config_path: process.config_path,
            endpoints: launch.endpoints,
        })
    }
}
