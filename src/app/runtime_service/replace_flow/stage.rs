use super::ports::assign_ephemeral_inbound_ports;
use super::*;

impl<'a> RuntimeService<'a> {
    pub(super) async fn stage_replacement_runtime(
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
                let failed_at = now_string();
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
                self.context
                    .db
                    .update_runtime_session_failure_tracking(
                        session_id,
                        None,
                        Some(&failed_at),
                        Some("replace_validation_failed"),
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
