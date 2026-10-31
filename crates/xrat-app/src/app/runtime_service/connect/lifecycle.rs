use super::*;

impl<'a> RuntimeService<'a> {
    pub fn new(context: &'a AppContext) -> Self {
        Self::with_process_ports(
            context,
            xrat_support::readiness::RuntimeProcessPorts::default(),
        )
    }

    pub fn with_process_ports(
        context: &'a AppContext,
        process_ports: xrat_support::readiness::RuntimeProcessPorts,
    ) -> Self {
        Self {
            context,
            rollback_context: None,
            process_ports,
        }
    }

    pub(crate) fn with_rollback_context(mut self, context: &'a AppContext) -> Self {
        self.rollback_context = Some(context);
        self
    }

    #[tracing::instrument(skip_all)]
    pub async fn disconnect(&self) -> crate::app::Result<DisconnectResult> {
        self.context.require_local_runtime()?;
        let session = self.context.db.get_running_runtime_session().await?;
        let stopped_session =
            stop_active_session(self.context, self.process_ports.signals.as_ref()).await?;
        if let Some(session) = session {
            self.cleanup_session_tun(session.id)?;
        } else {
            self.cleanup_inactive_tun_ownership().await?;
        }
        if self.context.app_config.runtime.tun.kill_switch
            != crate::app::config::TunKillSwitchMode::Strict
        {
            self.disengage_killswitch().await?;
        }
        Ok(DisconnectResult { stopped_session })
    }
}
