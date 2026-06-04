use super::*;

impl<'a> RuntimeService<'a> {
    pub async fn status(&self) -> crate::app::Result<RuntimeStatusSnapshot> {
        let active_state = self.active_session_state().await?;
        let latest = self.context.db.get_latest_runtime_session().await?;
        let session_config = match latest.as_ref().and_then(|session| session.config_id) {
            Some(config_id) => self.context.db.get_config_by_id(config_id).await?,
            None => None,
        };
        let active_config = self.context.db.get_active_config().await?;
        let pid_running = latest.as_ref().is_some_and(runtime_session_is_alive);
        let inbound_health = match &latest {
            Some(session) => check_runtime_inbounds(session, pid_running).await,
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
        active_session_state(self.context).await
    }
}
