use super::super::*;

#[derive(Clone, Copy)]
pub(crate) enum StopTransition<'a> {
    ManualDisconnect,
    DaemonShutdown {
        owner_instance_id: &'a str,
        restore_on_start: bool,
    },
}

pub(crate) async fn stop_active_session_with_transition(
    context: &AppContext,
    signals: &dyn xrat_support::signals::ProcessSignals,
    transition: StopTransition<'_>,
) -> crate::app::Result<bool> {
    let Some(session) = context.db.get_running_runtime_session().await? else {
        context.db.clear_active_config().await?;
        return Ok(false);
    };
    if let StopTransition::DaemonShutdown {
        owner_instance_id,
        restore_on_start,
    } = transition
    {
        context
            .db
            .update_runtime_session_transition_metadata(
                session.id,
                Some("daemon"),
                Some(owner_instance_id),
                Some(if restore_on_start {
                    REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED
                } else {
                    REASON_DAEMON_SHUTDOWN
                }),
                Some(if restore_on_start {
                    "daemon shutdown requested startup restoration"
                } else {
                    "daemon shutdown disconnected the runtime"
                }),
                Some("daemon"),
            )
            .await?;
    }
    stop_session(context, &session, signals).await?;
    if matches!(transition, StopTransition::ManualDisconnect) {
        context
            .db
            .update_runtime_session_transition_metadata(
                session.id,
                Some("cli"),
                None,
                Some(REASON_MANUAL_DISCONNECT),
                Some("runtime disconnect request succeeded"),
                Some("cli"),
            )
            .await?;
    }
    context.db.clear_active_config().await?;
    Ok(true)
}

pub(crate) async fn stop_session(
    context: &AppContext,
    session: &RuntimeSessionRecord,
    signals: &dyn xrat_support::signals::ProcessSignals,
) -> crate::app::Result<()> {
    #[cfg(windows)]
    if xrat_support::windows::current_user_sid()?.as_str() == "S-1-5-18"
        && session
            .process_id
            .is_some_and(|pid| signals.is_running(pid))
        && !crate::app::runtime_service::reattach::verified_system_session(context, session)
    {
        return Err(AppError::InvalidArgument(
            "refusing privileged termination of an unverified persisted runtime process; reconcile stale session state before retrying".into(),
        ));
    }
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
        let outcome = xray_runtime::terminate_process_gracefully_with_signals(
            pid,
            SHUTDOWN_TIMEOUT,
            signals,
        )?;
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
    Ok(())
}
