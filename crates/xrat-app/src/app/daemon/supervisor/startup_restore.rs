use super::*;

pub(super) const STARTUP_RESTORE_INITIAL_RETRY_SECONDS: u64 = 5;
const STARTUP_RESTORE_MAX_RETRY_SECONDS: u64 = 300;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StartupRestoreKind {
    GracefulShutdown,
    StalePid,
}

impl StartupRestoreKind {
    fn pending_reason(self) -> &'static str {
        match self {
            Self::GracefulShutdown => REASON_DAEMON_STARTUP_RESTORE_PENDING,
            Self::StalePid => REASON_DAEMON_STALE_PID_RESTORE_PENDING,
        }
    }

    fn success_event(self) -> &'static str {
        match self {
            Self::GracefulShutdown => "daemon_startup_connection_restored",
            Self::StalePid => "daemon_restart_stale_pid_recovered",
        }
    }

    fn failure_event(self) -> &'static str {
        match self {
            Self::GracefulShutdown => "daemon_startup_connection_restore_failed",
            Self::StalePid => "daemon_restart_stale_pid_recovery_failed",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct PendingStartupRestore {
    pub(super) config_id: ConfigId,
    pub(super) source_session_id: i64,
    pub(super) kind: StartupRestoreKind,
    pub(super) retry_at: time::Instant,
    pub(super) next_retry_seconds: u64,
}

pub(super) async fn load_stale_pid_restore(
    context: &AppContext,
    config_id: ConfigId,
) -> Option<PendingStartupRestore> {
    let session = match context
        .db
        .get_latest_runtime_session_for_config(config_id)
        .await
    {
        Ok(Some(session)) => session,
        Ok(None) => return None,
        Err(error) => {
            tracing::warn!(%error, config_id = config_id.0, "could not load stale runtime session for startup recovery");
            return None;
        }
    };
    pending_restore_from_session(context, session, StartupRestoreKind::StalePid).await
}

pub(super) async fn load_persisted_startup_restore(
    context: &AppContext,
    allow_stale_pid_recovery: bool,
) -> Option<PendingStartupRestore> {
    let session = match context.db.get_latest_runtime_session().await {
        Ok(Some(session)) => session,
        Ok(None) => return None,
        Err(error) => {
            tracing::warn!(%error, "could not load persisted startup restore intent");
            return None;
        }
    };
    let reason = session.last_transition_reason_code.as_deref();
    let kind = if allow_stale_pid_recovery
        && matches!(
            reason,
            Some(
                REASON_DAEMON_STALE_PID_RESTORE_PENDING
                    | "daemon_restart_reattach_rejected_pid_missing"
            )
        ) {
        StartupRestoreKind::StalePid
    } else if context.app_config.runtime.restore_connection_on_start
        && (reason == Some(REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED)
            || reason == Some(REASON_DAEMON_STARTUP_RESTORE_PENDING))
    {
        StartupRestoreKind::GracefulShutdown
    } else {
        return None;
    };
    pending_restore_from_session(context, session, kind).await
}

async fn pending_restore_from_session(
    context: &AppContext,
    session: xrat_db::RuntimeSessionRecord,
    kind: StartupRestoreKind,
) -> Option<PendingStartupRestore> {
    if matches!(
        session.status,
        xrat_db::RuntimeSessionStatus::Starting
            | xrat_db::RuntimeSessionStatus::Running
            | xrat_db::RuntimeSessionStatus::Stopping
    ) {
        return None;
    }
    let config_id = session.config_id?;
    let config = match context.db.get_config_by_id(config_id).await {
        Ok(Some(config)) => config,
        Ok(None) => return None,
        Err(error) => {
            tracing::warn!(%error, config_id = config_id.0, "could not load config for startup restore");
            return None;
        }
    };
    if !config.is_enabled || config.is_deleted {
        return None;
    }
    Some(PendingStartupRestore {
        config_id,
        source_session_id: session.id,
        kind,
        retry_at: time::Instant::now(),
        next_retry_seconds: STARTUP_RESTORE_INITIAL_RETRY_SECONDS,
    })
}

pub(super) async fn attempt_startup_restore(
    state: &mut SupervisorState,
    context: &AppContext,
    daemon_instance_id: &str,
    pending: &mut Option<PendingStartupRestore>,
) {
    let Some(restore) = pending.as_mut() else {
        return;
    };
    let config_id = restore.config_id;
    match context.db.get_config_by_id(config_id).await {
        Ok(Some(config)) if config.is_enabled && !config.is_deleted => {}
        Ok(_) => {
            mark_restore_unavailable(context, daemon_instance_id, restore).await;
            *pending = None;
            return;
        }
        Err(error) => {
            schedule_restore_retry(context, daemon_instance_id, restore, error.to_string()).await;
            return;
        }
    }

    match RuntimeService::new(context)
        .connect_for_startup_restore(
            ConnectRequest { config_id },
            daemon_instance_id,
            restore.kind.pending_reason(),
        )
        .await
    {
        Ok(result) => {
            if result.config.is_xray_profile() {
                state.pause_rotation_for_xray_profile();
            }
            events::record(
                &context.db,
                events::LEVEL_INFO,
                events::SOURCE_RUNTIME,
                restore.kind.success_event(),
                format!("Reconnected config {config_id} during daemon startup"),
                Some(config_id),
                Some(result.session_id),
                None,
            )
            .await;
            *pending = None;
        }
        Err(error) => {
            schedule_restore_retry(context, daemon_instance_id, restore, error.to_string()).await;
        }
    }
}

pub(super) async fn schedule_restore_retry(
    context: &AppContext,
    daemon_instance_id: &str,
    restore: &mut PendingStartupRestore,
    error: String,
) {
    let session_id = latest_restore_session_id(context, restore).await;
    if let Err(metadata_error) = context
        .db
        .update_runtime_session_transition_metadata(
            session_id,
            Some("daemon"),
            Some(daemon_instance_id),
            Some(restore.kind.pending_reason()),
            Some("startup connection attempt failed; retry remains pending"),
            Some("daemon"),
        )
        .await
    {
        tracing::warn!(%metadata_error, session_id, "could not persist startup restore retry state");
    }
    events::record(
        &context.db,
        events::LEVEL_WARN,
        events::SOURCE_RUNTIME,
        restore.kind.failure_event(),
        format!(
            "Could not reconnect config {}; retrying in {} seconds",
            restore.config_id, restore.next_retry_seconds
        ),
        Some(restore.config_id),
        Some(session_id),
        Some(error),
    )
    .await;
    restore.retry_at = time::Instant::now() + Duration::from_secs(restore.next_retry_seconds);
    restore.next_retry_seconds = next_startup_restore_retry_delay_secs(restore.next_retry_seconds);
}

pub(super) fn next_startup_restore_retry_delay_secs(previous_seconds: u64) -> u64 {
    previous_seconds
        .saturating_mul(2)
        .min(STARTUP_RESTORE_MAX_RETRY_SECONDS)
}

async fn latest_restore_session_id(context: &AppContext, restore: &PendingStartupRestore) -> i64 {
    match context.db.get_latest_runtime_session().await {
        Ok(Some(session)) if session.config_id == Some(restore.config_id) => session.id,
        _ => restore.source_session_id,
    }
}

async fn mark_restore_unavailable(
    context: &AppContext,
    daemon_instance_id: &str,
    restore: &PendingStartupRestore,
) {
    let session_id = latest_restore_session_id(context, restore).await;
    if let Err(error) = context
        .db
        .update_runtime_session_transition_metadata(
            session_id,
            Some("daemon"),
            Some(daemon_instance_id),
            Some(REASON_DAEMON_STARTUP_RESTORE_UNAVAILABLE),
            Some("saved config is missing, disabled, or deleted"),
            Some("daemon"),
        )
        .await
    {
        tracing::warn!(%error, session_id, "could not clear unavailable startup restore state");
    }
}
