use super::*;

pub struct DisconnectOutcome {
    pub session_id: Option<i64>,
    pub result: Result<DisconnectResult, AppError>,
}

pub struct RuntimeTransitionService<'a> {
    pub(super) context: &'a AppContext,
    pub(super) instance_id: &'a str,
}

impl<'a> RuntimeTransitionService<'a> {
    pub fn new(context: &'a AppContext, instance_id: &'a str) -> Self {
        Self {
            context,
            instance_id,
        }
    }
    pub async fn connect(&self, request: ConnectRequest) -> Result<ConnectResult, AppError> {
        let config_id = request.config_id;
        match RuntimeService::new(self.context)
            .connect(ConnectRequest { config_id })
            .await
        {
            Ok(result) => {
                if let Err(error) = self
                    .context
                    .db
                    .update_runtime_session_transition_metadata(
                        result.session_id,
                        Some("daemon"),
                        Some(self.instance_id),
                        Some("manual_connect"),
                        Some("daemon runtime connect request succeeded"),
                        Some("daemon"),
                    )
                    .await
                {
                    tracing::warn!(session_id = result.session_id, config_id = config_id.0, %error, "runtime connect metadata update failed");
                }
                crate::app::events::record(
                    &self.context.db,
                    crate::app::events::LEVEL_INFO,
                    crate::app::events::SOURCE_RUNTIME,
                    "connect",
                    format!("Connected config {}", result.config.id),
                    Some(result.config.id),
                    Some(result.session_id),
                    None,
                )
                .await;
                Ok(result)
            }
            Err(err) => {
                crate::app::events::record(
                    &self.context.db,
                    crate::app::events::LEVEL_ERROR,
                    crate::app::events::SOURCE_RUNTIME,
                    "connect_failed",
                    format!("Connect failed for config {config_id}: {err}"),
                    Some(config_id),
                    None,
                    None,
                )
                .await;
                Err(err)
            }
        }
    }
    pub async fn disconnect(&self) -> DisconnectOutcome {
        let active_session_id = self.context
        .db
        .get_running_runtime_session()
        .await
        .unwrap_or_else(|error| {
            tracing::debug!(operation = "runtime_disconnect_session", %error, "failed to load optional supervisor state");
            None
        })
        .map(|session| session.id);
        let result = match RuntimeService::new(self.context).disconnect().await {
            Ok(result) => {
                if result.stopped_session
                    && let Some(session_id) = active_session_id
                    && let Err(error) = self
                        .context
                        .db
                        .update_runtime_session_transition_metadata(
                            session_id,
                            Some("daemon"),
                            Some(self.instance_id),
                            Some("manual_disconnect"),
                            Some("daemon runtime disconnect request succeeded"),
                            Some("daemon"),
                        )
                        .await
                {
                    tracing::debug!(operation = "runtime_disconnect_metadata", session_id = session_id, %error, "failed to persist supervisor metadata");
                }
                if result.stopped_session {
                    crate::app::events::record(
                        &self.context.db,
                        crate::app::events::LEVEL_INFO,
                        crate::app::events::SOURCE_RUNTIME,
                        "disconnect",
                        "Disconnected managed runtime",
                        None,
                        active_session_id,
                        None,
                    )
                    .await;
                }
                Ok(result)
            }
            Err(err) => Err(err),
        };
        DisconnectOutcome {
            session_id: active_session_id,
            result,
        }
    }
    pub async fn shutdown(context: &AppContext, daemon_instance_id: &str) -> bool {
        let runtime_disconnected = RuntimeService::new(context)
            .disconnect_for_daemon_shutdown(
                daemon_instance_id,
                context.app_config.runtime.restore_connection_on_start,
            )
            .await
            .map(|result| result.stopped_session)
            .unwrap_or_else(|error| {
                tracing::debug!(operation = "daemon_shutdown_disconnect", %error, "runtime disconnect failed during shutdown");
                false
            });
        crate::app::events::record(
            &context.db,
            crate::app::events::LEVEL_INFO,
            crate::app::events::SOURCE_DAEMON,
            "daemon_stopped",
            "Daemon supervisor stopped",
            None,
            None,
            None,
        )
        .await;
        runtime_disconnected
    }
    pub async fn record_health_failure(
        &self,
        session: &xrat_db::RuntimeSessionRecord,
        cooldown_secs: u64,
        reason: &str,
    ) {
        let failed_at = xrat_support::time::now_epoch_seconds();
        let cooldown_until = (failed_at + cooldown_secs).to_string();
        let failed_at = failed_at.to_string();
        if let Err(error) = self
            .context
            .db
            .update_runtime_session_transition_metadata(
                session.id,
                Some("daemon"),
                Some(self.instance_id),
                Some(reason),
                Some("runtime health check requested recovery"),
                Some("daemon"),
            )
            .await
        {
            tracing::debug!(operation = "health_transition_metadata", session_id = session.id, %error, "failed to persist supervisor metadata");
        }
        if let Err(error) = self
            .context
            .db
            .update_runtime_session_failure_tracking(
                session.id,
                Some(&cooldown_until),
                Some(&failed_at),
                Some(reason),
            )
            .await
        {
            tracing::debug!(operation = "health_failure_tracking", session_id = session.id, %error, "failed to persist supervisor metadata");
        }
    }
}
