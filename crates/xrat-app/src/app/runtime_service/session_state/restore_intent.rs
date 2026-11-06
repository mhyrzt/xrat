use crate::app::context::AppContext;

pub(crate) const REASON_DAEMON_SHUTDOWN: &str = "daemon_shutdown";
pub(crate) const REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED: &str =
    "daemon_shutdown_restore_requested";
pub(crate) const REASON_DAEMON_STARTUP_RESTORE_PENDING: &str = "daemon_startup_restore_pending";
pub(crate) const REASON_DAEMON_STALE_PID_RESTORE_PENDING: &str = "daemon_stale_pid_restore_pending";
pub(crate) const REASON_DAEMON_STARTUP_RESTORE_SUCCEEDED: &str = "daemon_startup_restore_succeeded";
pub(crate) const REASON_DAEMON_STARTUP_RESTORE_UNAVAILABLE: &str =
    "daemon_startup_restore_unavailable";

pub(crate) const REASON_MANUAL_CONNECT: &str = "manual_connect";
pub(crate) const REASON_MANUAL_DISCONNECT: &str = "manual_disconnect";

pub(crate) fn is_startup_restore_reason(reason: Option<&str>) -> bool {
    matches!(
        reason,
        Some(
            REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED
                | REASON_DAEMON_STARTUP_RESTORE_PENDING
                | REASON_DAEMON_STALE_PID_RESTORE_PENDING
                | "daemon_restart_reattach_rejected_pid_missing"
        )
    )
}

pub(crate) async fn cancel_pending_startup_restore(
    context: &AppContext,
    owner_kind: &str,
    owner_instance_id: Option<&str>,
    reason_code: &str,
    reason_detail: &str,
    transition_origin: &str,
) -> crate::app::Result<bool> {
    let Some(session) = context.db.get_latest_runtime_session().await? else {
        return Ok(false);
    };
    if !is_startup_restore_reason(session.last_transition_reason_code.as_deref()) {
        return Ok(false);
    }

    context
        .db
        .update_runtime_session_transition_metadata(
            session.id,
            Some(owner_kind),
            owner_instance_id,
            Some(reason_code),
            Some(reason_detail),
            Some(transition_origin),
        )
        .await?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::runtime_service::RuntimeService;
    use xrat_db::{RuntimeSessionInsert, RuntimeSessionStatus};

    async fn insert_session(context: &AppContext, status: RuntimeSessionStatus) -> i64 {
        context
            .db
            .insert_runtime_session(&RuntimeSessionInsert {
                config_id: None,
                status,
                socks_host: None,
                socks_port: None,
                http_host: None,
                http_port: None,
                shadowsocks_host: None,
                shadowsocks_port: None,
                process_id: None,
                failure_reason: None,
                started_at: None,
                stopped_at: None,
            })
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn daemon_shutdown_records_restore_intent_before_stopping_runtime() {
        let (mut context, _root) =
            crate::app::tests::TestAppBuilder::new("shutdown-restore-intent")
                .build_with_root()
                .await;
        context.app_config.runtime.restore_connection_on_start = true;
        let session_id = insert_session(&context, RuntimeSessionStatus::Running).await;

        let result = RuntimeService::new(&context)
            .disconnect_for_daemon_shutdown("daemon-test", true)
            .await
            .unwrap();

        assert!(result.stopped_session);
        let session = context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(session.id, session_id);
        assert_eq!(session.status, RuntimeSessionStatus::Stopped);
        assert_eq!(
            session.last_transition_reason_code.as_deref(),
            Some(REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED)
        );
        assert_eq!(session.last_transition_origin.as_deref(), Some("daemon"));
    }

    #[tokio::test]
    async fn daemon_shutdown_without_opt_in_does_not_request_restore() {
        let (context, _root) = crate::app::tests::TestAppBuilder::new("shutdown-without-restore")
            .build_with_root()
            .await;
        let session_id = insert_session(&context, RuntimeSessionStatus::Running).await;

        let result = RuntimeService::new(&context)
            .disconnect_for_daemon_shutdown("daemon-test", false)
            .await
            .unwrap();

        assert!(result.stopped_session);
        let session = context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(session.id, session_id);
        assert_eq!(session.status, RuntimeSessionStatus::Stopped);
        assert_eq!(
            session.last_transition_reason_code.as_deref(),
            Some(REASON_DAEMON_SHUTDOWN)
        );
    }

    #[tokio::test]
    async fn manual_disconnect_cancels_persisted_restore_without_active_process() {
        let (context, _root) = crate::app::tests::TestAppBuilder::new("manual-cancel-restore")
            .build_with_root()
            .await;
        let session_id = insert_session(&context, RuntimeSessionStatus::Stopped).await;
        context
            .db
            .update_runtime_session_transition_metadata(
                session_id,
                Some("daemon"),
                Some("previous-daemon"),
                Some(REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED),
                None,
                Some("daemon"),
            )
            .await
            .unwrap();

        let result = RuntimeService::new(&context).disconnect().await.unwrap();

        assert!(!result.stopped_session);
        let session = context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            session.last_transition_reason_code.as_deref(),
            Some(REASON_MANUAL_DISCONNECT)
        );
    }
}
