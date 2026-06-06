use super::*;

mod process;

const REASON_REATTACH_REJECTED_PID_MISSING: &str = "daemon_restart_reattach_rejected_pid_missing";
const REASON_REATTACH_REJECTED_EXEC_MISMATCH: &str =
    "daemon_restart_reattach_rejected_exec_mismatch";
const REASON_REATTACH_REJECTED_CMDLINE_MISMATCH: &str =
    "daemon_restart_reattach_rejected_cmdline_mismatch";

impl<'a> RuntimeService<'a> {
    /// Reconcile a persisted runtime session against live processes on daemon
    /// start. Returns `Some(config_id)` when a session was rejected because its
    /// proxy process is gone (for example after a reboot) but the persisted
    /// config is still usable, signalling the caller to relaunch it.
    pub async fn reconcile_reattach_on_daemon_start(
        &self,
        daemon_instance_id: &str,
    ) -> crate::app::Result<Option<i64>> {
        self.reconcile_reattach_with_inspector(&process::SystemProcessInspector, daemon_instance_id)
            .await
    }

    pub(super) async fn reconcile_reattach_with_inspector(
        &self,
        inspector: &dyn ProcessInspector,
        daemon_instance_id: &str,
    ) -> crate::app::Result<Option<i64>> {
        let Some(session) = self.context.db.get_running_runtime_session().await? else {
            return Ok(None);
        };

        if validate_reattach_session(self.context, &session, inspector) {
            self.context
                .db
                .update_runtime_session_transition_metadata(
                    session.id,
                    Some("daemon"),
                    Some(daemon_instance_id),
                    Some("daemon_restart_reattach_ok"),
                    None,
                    Some("daemon"),
                )
                .await?;
            return Ok(None);
        }

        let reject_reason = reattach_reject_reason(self.context, &session, inspector);
        self.context
            .db
            .update_runtime_session_state(
                session.id,
                RuntimeSessionStatus::Failed,
                None,
                None,
                Some(&now_string()),
                Some(reject_reason),
            )
            .await?;
        self.context
            .db
            .update_runtime_session_transition_metadata(
                session.id,
                Some("daemon"),
                Some(daemon_instance_id),
                Some(reject_reason),
                None,
                Some("daemon"),
            )
            .await?;
        self.context.db.clear_active_config().await?;

        // A missing PID means the proxy process is genuinely gone (the common
        // reboot case), so it is safe to relaunch from the persisted config. An
        // exec/cmdline mismatch means a different process now owns that PID, so
        // we do not auto-launch over it.
        if reject_reason == REASON_REATTACH_REJECTED_PID_MISSING
            && let Some(config_id) = session.config_id
            && let Some(config) = self.context.db.get_config_by_id(config_id).await?
            && config.is_enabled
            && !config.is_deleted
        {
            return Ok(Some(config_id));
        }

        Ok(None)
    }
}

pub(super) trait ProcessInspector: Sync {
    fn is_running(&self, pid: i64) -> bool;
    fn exec_matches_runtime_engine(&self, context: &AppContext, pid: i64) -> bool;
    fn cmdline_matches_session_config(
        &self,
        context: &AppContext,
        pid: i64,
        session_id: i64,
    ) -> bool;
}

fn reattach_reject_reason(
    context: &AppContext,
    session: &RuntimeSessionRecord,
    inspector: &dyn ProcessInspector,
) -> &'static str {
    let Some(pid) = session.process_id else {
        return REASON_REATTACH_REJECTED_PID_MISSING;
    };
    if !inspector.is_running(pid) {
        return REASON_REATTACH_REJECTED_PID_MISSING;
    }
    if !inspector.exec_matches_runtime_engine(context, pid) {
        return REASON_REATTACH_REJECTED_EXEC_MISMATCH;
    }
    if !inspector.cmdline_matches_session_config(context, pid, session.id) {
        return REASON_REATTACH_REJECTED_CMDLINE_MISMATCH;
    }
    REASON_REATTACH_REJECTED_PID_MISSING
}

fn validate_reattach_session(
    context: &AppContext,
    session: &RuntimeSessionRecord,
    inspector: &dyn ProcessInspector,
) -> bool {
    let Some(pid) = session.process_id else {
        return false;
    };

    inspector.is_running(pid)
        && inspector.exec_matches_runtime_engine(context, pid)
        && inspector.cmdline_matches_session_config(context, pid, session.id)
}
