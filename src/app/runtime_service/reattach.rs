use super::*;

const REASON_REATTACH_REJECTED_PID_MISSING: &str = "daemon_restart_reattach_rejected_pid_missing";
const REASON_REATTACH_REJECTED_EXEC_MISMATCH: &str =
    "daemon_restart_reattach_rejected_exec_mismatch";
const REASON_REATTACH_REJECTED_CMDLINE_MISMATCH: &str =
    "daemon_restart_reattach_rejected_cmdline_mismatch";

impl<'a> RuntimeService<'a> {
    pub async fn reconcile_reattach_on_daemon_start(&self) -> crate::app::Result<()> {
        self.reconcile_reattach_with_inspector(&SystemProcessInspector)
            .await
    }

    pub(super) async fn reconcile_reattach_with_inspector(
        &self,
        inspector: &dyn ProcessInspector,
    ) -> crate::app::Result<()> {
        let Some(session) = self.context.db.get_running_runtime_session().await? else {
            return Ok(());
        };

        if !validate_reattach_session(self.context, &session, inspector) {
            self.context
                .db
                .update_runtime_session_state(
                    session.id,
                    RuntimeSessionStatus::Failed,
                    None,
                    None,
                    Some(&now_string()),
                    Some(reattach_reject_reason(self.context, &session, inspector)),
                )
                .await?;
            self.context.db.clear_active_config().await?;
        }

        Ok(())
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

struct SystemProcessInspector;

impl ProcessInspector for SystemProcessInspector {
    fn is_running(&self, pid: i64) -> bool {
        xray_runtime::process_is_running(pid)
    }

    fn exec_matches_runtime_engine(&self, context: &AppContext, pid: i64) -> bool {
        process_exec_matches_runtime_engine(context, pid)
    }

    fn cmdline_matches_session_config(
        &self,
        context: &AppContext,
        pid: i64,
        session_id: i64,
    ) -> bool {
        process_cmdline_matches_session_config(context, pid, session_id)
    }
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

#[cfg(unix)]
fn process_exec_matches_runtime_engine(context: &AppContext, pid: i64) -> bool {
    let process_exe = std::fs::read_link(format!("/proc/{pid}/exe")).ok();
    let Some(process_exe) = process_exe else {
        return false;
    };

    let expected = match context.app_config.runtime.engine.as_str() {
        "xray" => &context.runtime_paths.xray_path,
        "v2ray" => &context.runtime_paths.v2ray_path,
        "sing-box" => &context.runtime_paths.sing_box_path,
        _ => &context.runtime_paths.xray_path,
    };
    process_exe
        .file_name()
        .map(|name| name == expected.file_name().unwrap_or_default())
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn process_exec_matches_runtime_engine(_context: &AppContext, _pid: i64) -> bool {
    false
}

#[cfg(unix)]
fn process_cmdline_matches_session_config(context: &AppContext, pid: i64, session_id: i64) -> bool {
    let cmdline_bytes = std::fs::read(format!("/proc/{pid}/cmdline")).ok();
    let Some(cmdline_bytes) = cmdline_bytes else {
        return false;
    };
    let expected = context
        .runtime_paths
        .runtime_dir
        .join(format!("session-{session_id}.json"));
    let expected = expected.to_string_lossy();
    let cmdline = cmdline_bytes
        .split(|byte| *byte == 0)
        .filter(|segment| !segment.is_empty())
        .map(|segment| String::from_utf8_lossy(segment).to_string())
        .collect::<Vec<_>>();
    cmdline.iter().any(|arg| arg.contains(expected.as_ref()))
}

#[cfg(not(unix))]
fn process_cmdline_matches_session_config(
    _context: &AppContext,
    _pid: i64,
    _session_id: i64,
) -> bool {
    false
}
