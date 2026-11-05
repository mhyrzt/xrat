use super::*;

impl<'a> RuntimeService<'a> {
    #[tracing::instrument(skip_all)]
    pub async fn replace(&self, request: ReplaceRequest) -> crate::app::Result<ReplaceResult> {
        self.context.require_local_runtime()?;
        let active = match self.active_session_state().await? {
            ActiveSessionState::Running(session) => session,
            ActiveSessionState::Stale(_) | ActiveSessionState::None => {
                let next_config_id = self.resolve_initial_rotation_candidate_id(&request).await?;
                let next_config = self
                    .context
                    .db
                    .get_config_by_id(next_config_id)
                    .await?
                    .ok_or_else(|| {
                        AppError::InvalidArgument(format!("config {next_config_id} was not found"))
                    })?;
                let launch = self.resolve_launch(&next_config).await?;
                preflight_runtime_with_spawner(
                    &launch,
                    &self.context.runtime_paths.runtime_dir,
                    self.process_ports.spawner.clone(),
                )?;
                let connected = self
                    .connect(ConnectRequest {
                        config_id: next_config_id,
                    })
                    .await?;
                self.context
                    .db
                    .update_runtime_session_transition_metadata(
                        connected.session_id,
                        None,
                        None,
                        Some("replace_commit_success"),
                        Some("runtime rotation started new session"),
                        Some("daemon"),
                    )
                    .await?;
                return Ok(ReplaceResult {
                    old_session_id: None,
                    new_config_id: connected.config.id,
                    new_session_id: connected.session_id,
                    new_pid: connected.pid,
                });
            }
        };
        let next_config_id = match self.resolve_replace_candidate_id(&active, &request).await {
            Ok(config_id) => config_id,
            Err(err) => {
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

        let next_config = self
            .context
            .db
            .get_config_by_id(next_config_id)
            .await?
            .ok_or_else(|| {
                AppError::InvalidArgument(format!("config {next_config_id} was not found"))
            })?;
        let launch = self.resolve_launch(&next_config).await?;
        if self.context.app_config.runtime.tun.enabled {
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
        preflight_runtime_with_spawner(
            &launch,
            &self.context.runtime_paths.runtime_dir,
            self.process_ports.spawner.clone(),
        )?;

        stop_session(self.context, &active, self.process_ports.signals.as_ref()).await?;
        self.context.db.clear_active_config().await?;

        if self.context.app_config.runtime.tun.enabled
            && let Err(error) = self.cleanup_stale_tun_interface()
        {
            return Err(self.rollback_runtime_error(active.config_id, error).await);
        }

        let staged = self.stage_replacement_runtime(next_config, launch).await;
        let (next_config_id, session_id, new_pid) = match staged {
            Ok(value) => value,
            Err(err) => {
                self.context.db.clear_active_config().await?;
                return Err(self.rollback_runtime_error(active.config_id, err).await);
            }
        };

        self.context.db.set_active_config(next_config_id).await?;
        self.context
            .db
            .update_runtime_session_transition_metadata(
                session_id,
                None,
                None,
                Some("replace_commit_success"),
                Some("runtime replace handoff completed"),
                Some("daemon"),
            )
            .await?;

        Ok(ReplaceResult {
            old_session_id: Some(active.id),
            new_config_id: next_config_id,
            new_session_id: session_id,
            new_pid,
        })
    }
}
