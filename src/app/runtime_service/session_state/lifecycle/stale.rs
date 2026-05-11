use super::super::*;

pub(super) async fn mark_session_stale(
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
    let transition_origin = session
        .owner_kind
        .as_deref()
        .filter(|origin| !origin.is_empty())
        .unwrap_or("daemon");
    if let Some(reason_code) = reason_code {
        let failed_at = now_string();
        context
            .db
            .update_runtime_session_transition_metadata(
                session.id,
                None,
                None,
                Some(reason_code),
                Some(stale_session_reason(session)),
                Some(transition_origin),
            )
            .await?;
        if reason_code == "process_exit_unexpected" {
            context
                .db
                .update_runtime_session_failure_tracking(
                    session.id,
                    None,
                    Some(&failed_at),
                    Some(reason_code),
                )
                .await?;
        }
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
