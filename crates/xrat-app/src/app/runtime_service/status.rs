use super::*;

impl<'a> RuntimeService<'a> {
    pub async fn status(&self) -> crate::app::Result<RuntimeStatusSnapshot> {
        #[cfg(any(windows, test))]
        if let Some(client) = &self.context.machine_client {
            return Ok(client.overview().await?.runtime);
        }
        let active_state = self.active_session_state().await?;
        let latest = match &active_state {
            ActiveSessionState::Running(session) | ActiveSessionState::Stale(session) => {
                Some(session.clone())
            }
            ActiveSessionState::None => self.context.db.get_latest_runtime_session().await?,
        };
        let session_config = match latest.as_ref().and_then(|session| session.config_id) {
            Some(config_id) => self.context.db.get_config_by_id(config_id).await?,
            None => None,
        };
        let active_config = self.context.db.get_active_config().await?;
        let pid_running = latest.as_ref().is_some_and(|session| {
            runtime_session_is_alive(session, self.process_ports.signals.as_ref())
        });
        let inbound_health = match &latest {
            Some(session) => {
                check_runtime_inbounds(session, pid_running, self.process_ports.connector.as_ref())
                    .await
            }
            None => RuntimeInboundHealth::default(),
        };
        let status = runtime_status_label(&latest, &active_state, pid_running, &inbound_health);

        Ok(RuntimeStatusSnapshot {
            status,
            session: latest,
            session_config,
            active_config,
            pid_running,
            inbound_health,
            database_label: self.context.runtime_paths.database_label.clone(),
        })
    }

    pub async fn active_session_state(&self) -> crate::app::Result<ActiveSessionState> {
        self.context.require_local_runtime()?;
        let state = active_session_state(self.context, self.process_ports.signals.as_ref()).await?;
        if let ActiveSessionState::Stale(session) = &state {
            self.cleanup_session_tun(session.id)?;
        }
        if matches!(state, ActiveSessionState::None) {
            self.cleanup_inactive_tun_ownership().await?;
        }
        Ok(state)
    }
}
