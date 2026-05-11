use super::*;

pub(crate) async fn active_session_state(
    context: &AppContext,
) -> crate::app::Result<ActiveSessionState> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        return Ok(ActiveSessionState::None);
    };

    if runtime_session_is_alive(&session) {
        return Ok(ActiveSessionState::Running(session));
    }

    mark_session_stale(context, &session).await?;
    Ok(ActiveSessionState::Stale(session))
}

pub(crate) async fn stop_active_session(context: &AppContext) -> crate::app::Result<bool> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        context.db.clear_active_config().await?;
        return Ok(false);
    };

    context
        .db
        .update_runtime_session_state(
            session.id,
            RuntimeSessionStatus::Stopping,
            None,
            None,
            None,
            None,
        )
        .await?;

    if let Some(pid) = session.process_id {
        let outcome = xray_runtime::terminate_process_gracefully(pid, SHUTDOWN_TIMEOUT)?;
        tracing::info!(
            session_id = session.id,
            pid,
            outcome = ?outcome,
            "runtime process termination completed"
        );
    } else {
        tracing::warn!(
            session_id = session.id,
            "runtime session has no saved process id"
        );
    }

    context
        .db
        .mark_runtime_session_stopped(session.id, Some(&now_string()))
        .await?;
    context.db.clear_active_config().await?;
    Ok(true)
}

pub(crate) fn runtime_session_is_alive(session: &RuntimeSessionRecord) -> bool {
    session
        .process_id
        .map(xray_runtime::process_is_running)
        .unwrap_or(false)
}

async fn mark_session_stale(
    context: &AppContext,
    session: &RuntimeSessionRecord,
) -> crate::app::Result<()> {
    let terminal_status = match session.status {
        RuntimeSessionStatus::Stopping => RuntimeSessionStatus::Stopped,
        RuntimeSessionStatus::Starting | RuntimeSessionStatus::Running => {
            RuntimeSessionStatus::Failed
        }
        RuntimeSessionStatus::Stopped | RuntimeSessionStatus::Failed => return Ok(()),
    };

    context
        .db
        .update_runtime_session_state(
            session.id,
            terminal_status,
            None,
            None,
            Some(&now_string()),
            Some(stale_session_reason(session)),
        )
        .await?;
    let reason_code = match session.status {
        RuntimeSessionStatus::Starting | RuntimeSessionStatus::Running => {
            Some("process_exit_unexpected")
        }
        RuntimeSessionStatus::Stopping => Some("manual_disconnect"),
        RuntimeSessionStatus::Stopped | RuntimeSessionStatus::Failed => None,
    };
    if let Some(reason_code) = reason_code {
        context
            .db
            .update_runtime_session_transition_metadata(
                session.id,
                None,
                None,
                Some(reason_code),
                Some(stale_session_reason(session)),
            )
            .await?;
    }
    context.db.clear_active_config().await?;
    Ok(())
}

fn stale_session_reason(session: &RuntimeSessionRecord) -> &'static str {
    match session.status {
        RuntimeSessionStatus::Stopping => "runtime process disappeared while stopping",
        RuntimeSessionStatus::Starting | RuntimeSessionStatus::Running => {
            "runtime process is not running"
        }
        RuntimeSessionStatus::Stopped | RuntimeSessionStatus::Failed => "runtime session is closed",
    }
}
