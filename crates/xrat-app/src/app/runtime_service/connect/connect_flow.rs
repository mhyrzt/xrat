use super::*;

impl<'a> RuntimeService<'a> {
    #[tracing::instrument(skip_all, fields(config_id = request.config_id.0))]
    pub async fn connect(&self, request: ConnectRequest) -> crate::app::Result<ConnectResult> {
        self.context.require_local_runtime()?;
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

        let tun_enabled = self.context.app_config.runtime.tun.enabled;
        let (launch, active_session) = match self.active_session_state().await? {
            ActiveSessionState::Running(session) => {
                if !self.context.app_config.runtime.replace_active_session {
                    tracing::warn!(
                        session_id = session.id,
                        "active runtime session blocks connect"
                    );
                    return Err(AppError::RuntimeSessionAlreadyActive);
                }

                (self.resolve_launch(&config)?, Some(session))
            }
            ActiveSessionState::Stale(session) => {
                tracing::warn!(
                    session_id = session.id,
                    "stale runtime session was reconciled before connect"
                );
                (self.resolve_launch(&config)?, None)
            }
            ActiveSessionState::None => (self.resolve_launch(&config)?, None),
        };
        let previous_active_config_id = active_session.as_ref().and_then(|s| s.config_id);

        let retiring_tun = active_session.as_ref().and_then(|session| {
            tun_ownership::load_ownership(&self.context.runtime_paths.runtime_dir)
                .filter(|record| record.session_id == session.id)
        });
        let mut retiring_context = self.context.clone();
        if let Some(record) = &retiring_tun {
            retiring_context.app_config.runtime.tun.interface_name = record.interface_name.clone();
            RuntimeService::with_process_ports(&retiring_context, self.process_ports.clone())
                .verify_tun_interface()?;
        }

        if tun_enabled {
            self.preflight_tun_policy_rules()?;
            crate::app::tun_privileges::ensure_engine_capability_with_spawner(
                &launch.binary_path,
                self.process_ports.spawner.clone(),
            )?;
            if self.context.app_config.runtime.tun.split_mode
                != crate::app::config::TunSplitMode::All
            {
                crate::app::tun_privileges::ensure_engine_process_routing_capability_with_spawner(
                    &launch.binary_path,
                    self.process_ports.spawner.clone(),
                )?;
            }
            self.verify_tun_interface()?;
        }

        // Validate the replacement before tearing down a healthy session so a
        // failed preflight leaves the running runtime untouched.
        preflight_runtime_with_spawner(
            &launch,
            &self.context.runtime_paths.runtime_dir,
            self.process_ports.spawner.clone(),
        )?;

        if let Some(active) = &active_session {
            stop_session(self.context, active, self.process_ports.signals.as_ref()).await?;
            self.context.db.clear_active_config().await?;
        }

        if retiring_tun.is_some()
            && let Err(error) =
                RuntimeService::with_process_ports(&retiring_context, self.process_ports.clone())
                    .cleanup_stale_tun_interface()
        {
            return Err(self
                .rollback_runtime_error(previous_active_config_id, error)
                .await);
        }

        if !tun_enabled
            && self.context.app_config.runtime.tun.kill_switch
                != crate::app::config::TunKillSwitchMode::Strict
            && let Err(error) = self.disengage_killswitch().await
        {
            return Err(self
                .rollback_runtime_error(previous_active_config_id, error)
                .await);
        }

        if tun_enabled && let Err(error) = self.cleanup_stale_tun_interface() {
            return Err(self
                .rollback_runtime_error(previous_active_config_id, error)
                .await);
        }

        crate::app::runtime_service::log_retention::cleanup(self.context).await;
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

        if tun_enabled && let Err(error) = self.begin_tun_ownership(session_id).await {
            return Err(self
                .rollback_runtime_error(previous_active_config_id, error)
                .await);
        }

        let process = match spawn_runtime_with_ports(
            &launch,
            &self.context.runtime_paths.runtime_dir,
            session_id,
            self.process_ports.clone(),
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

                return Err(self
                    .rollback_runtime_error(
                        previous_active_config_id,
                        tun_startup_error(error, tun_enabled),
                    )
                    .await);
            }
        };

        if let Err(error) = self
            .finish_tun_startup(session_id, process.pid, &launch.allowed_endpoints)
            .await
        {
            return Err(self
                .rollback_runtime_error(previous_active_config_id, error)
                .await);
        }

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

/// Append TUN capability guidance to a startup failure when TUN capture is
/// enabled. An engine can pass native validation but still fail to start when it
/// cannot create the interface or system routes.
fn tun_startup_error(error: AppError, tun_enabled: bool) -> AppError {
    let message = error.to_string();
    let looks_like_startup =
        message.contains("exited during startup") || message.contains("startup timeout");
    if tun_enabled && looks_like_startup {
        return AppError::InvalidArgument(format!(
            "{message}; if this is a TUN capture failure, ensure capabilities with `xrat tun setup`"
        ));
    }
    error
}
