use super::*;

pub(super) const HEALTH_TICK_SECONDS: u64 = 15;
const STARTUP_RESTORE_TICK_SECONDS: u64 = 1;

/// How often the daemon checks whether any URL-backed subscription is due for an
/// automatic refresh. The per-subscription cadence is governed by
/// `[subscriptions].refresh_interval_hours`; this only bounds detection lag.
pub(super) const SUBSCRIPTION_REFRESH_TICK_SECONDS: u64 = 300;

pub async fn run(mut rx: mpsc::Receiver<SupervisorEvent>, mut context: AppContext) {
    run_inner(&mut rx, &mut context, true).await;
}

#[cfg(windows)]
pub async fn run_machine(mut rx: mpsc::Receiver<SupervisorEvent>, mut context: AppContext) {
    context.app_config.runtime.rotation.enabled = false;
    run_inner(&mut rx, &mut context, false).await;
}

async fn run_inner(
    rx: &mut mpsc::Receiver<SupervisorEvent>,
    context: &mut AppContext,
    recover_stale_session: bool,
) {
    let daemon_instance_id = uuid::Uuid::new_v4().to_string();
    let mut startup_restore = None;
    if recover_stale_session {
        match RuntimeService::new(context)
            .reconcile_reattach_on_daemon_start(&daemon_instance_id)
            .await
        {
            Ok(Some(config_id)) => {
                startup_restore = load_stale_pid_restore(context, config_id).await;
            }
            Ok(None) => {}
            Err(err) => tracing::warn!(error = %err, "daemon reattach reconciliation failed"),
        }
    }
    if startup_restore.is_none() {
        startup_restore = load_persisted_startup_restore(context, recover_stale_session).await;
    }
    let mut state = SupervisorState::new(daemon_instance_id.clone());
    state.rotation_enabled = context.app_config.runtime.rotation.enabled;
    state.rotation_interval_secs = context.app_config.runtime.rotation.interval_secs;
    state.health_trigger_enabled = context.app_config.runtime.rotation.health_trigger_enabled;
    state.health_failure_threshold = context
        .app_config
        .runtime
        .rotation
        .health_failure_threshold
        .max(1);
    state.cooldown_secs = context.app_config.runtime.rotation.cooldown_secs;
    if state.rotation_enabled {
        state.next_timer_epoch_secs = Some(now_epoch_seconds() + state.rotation_interval_secs);
    }
    if let Ok(Some(session)) = context.db.get_running_runtime_session().await
        && let Some(id) = session.config_id
        && context
            .db
            .get_config_by_id(id)
            .await
            .ok()
            .flatten()
            .is_some_and(|config| config.is_xray_profile())
    {
        state.pause_rotation_for_xray_profile();
    }
    let mut health_ticker = time::interval(Duration::from_secs(HEALTH_TICK_SECONDS));
    health_ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    let auto_refresh_enabled = context.app_config.subscriptions.auto_refresh;
    let refresh_in_progress = Arc::new(AtomicBool::new(false));
    let mut refresh_ticker = time::interval(Duration::from_secs(SUBSCRIPTION_REFRESH_TICK_SECONDS));
    refresh_ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    let mut restore_ticker = time::interval(Duration::from_secs(STARTUP_RESTORE_TICK_SECONDS));
    restore_ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    let (health_result_tx, mut health_result_rx) = mpsc::channel(8);

    loop {
        tokio::select! {
            _ = health_ticker.tick() => {
                handlers::handle_event_with_sender(
                    &mut state,
                    SupervisorEvent::HealthTick,
                    context,
                    &health_result_tx,
                ).await;
            }
            _ = refresh_ticker.tick(), if auto_refresh_enabled => {
                spawn_due_refresh(context, &refresh_in_progress);
            }
            _ = restore_ticker.tick(), if startup_restore.is_some() => {
                if startup_restore.as_ref().is_some_and(|restore| restore.retry_at <= time::Instant::now()) {
                    attempt_startup_restore(
                        &mut state,
                        context,
                        &daemon_instance_id,
                        &mut startup_restore,
                    )
                    .await;
                }
            }
            event = rx.recv() => {
                let Some(event) = event else {
                    break;
                };
                if matches!(
                    &event,
                    SupervisorEvent::RuntimeConnect { .. } | SupervisorEvent::RuntimeDisconnect { .. }
                ) {
                    startup_restore = None;
                }
                let shutdown = matches!(&event, SupervisorEvent::DaemonShutdown { .. });
                handlers::handle_event_with_sender(&mut state, event, context, &health_result_tx).await;
                if shutdown { break; }
            }
            event = health_result_rx.recv() => {
                let Some(event) = event else {
                    continue;
                };
                handlers::handle_event_with_sender(&mut state, event, context, &health_result_tx).await;
            }
        }
    }
}

/// Spawn a detached task that refreshes any due URL-backed subscriptions, so a
/// slow provider fetch never stalls the supervisor loop. The guard ensures only
/// one refresh batch runs at a time even if ticks pile up.
pub(super) fn spawn_due_refresh(context: &AppContext, refresh_in_progress: &Arc<AtomicBool>) {
    if refresh_in_progress.swap(true, Ordering::SeqCst) {
        return;
    }
    let context = context.clone();
    let guard = Arc::clone(refresh_in_progress);
    tokio::spawn(async move {
        let outcome = subscription_refresh::refresh_due(&context).await;
        if outcome.attempted > 0 {
            tracing::info!(
                attempted = outcome.attempted,
                succeeded = outcome.succeeded,
                failed = outcome.failed,
                "automatic subscription refresh tick completed"
            );
        }
        guard.store(false, Ordering::SeqCst);
    });
}

#[cfg(test)]
mod tests {
    use super::super::startup_restore::{
        STARTUP_RESTORE_INITIAL_RETRY_SECONDS, StartupRestoreKind,
        next_startup_restore_retry_delay_secs, schedule_restore_retry,
    };
    use super::*;
    use crate::app::runtime_service::REASON_MANUAL_DISCONNECT;

    async fn restore_config(context: &AppContext) -> ConfigId {
        let summary = context
            .db
            .import_nodes(
                &crate::app::tests::fixtures::test_source(),
                &[crate::app::tests::fixtures::test_node_with(
                    "restore.example.com",
                    "restore",
                )],
            )
            .await
            .unwrap();
        assert_eq!(summary.imported_configs, 1);
        context
            .db
            .list_configs(&Default::default())
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap()
            .id
    }

    async fn stopped_session(context: &AppContext, config_id: ConfigId, reason: &str) -> i64 {
        let session_id = context
            .db
            .insert_runtime_session(&xrat_db::RuntimeSessionInsert {
                config_id: Some(config_id),
                status: xrat_db::RuntimeSessionStatus::Stopped,
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
            .unwrap();
        context
            .db
            .update_runtime_session_transition_metadata(
                session_id,
                Some("daemon"),
                Some("previous-daemon"),
                Some(reason),
                None,
                Some("daemon"),
            )
            .await
            .unwrap();
        session_id
    }

    #[tokio::test]
    async fn graceful_shutdown_restore_requires_setting_and_not_manual_disconnect() {
        let (mut context, _root) = crate::app::tests::TestAppBuilder::new("startup-restore-opt-in")
            .build_with_root()
            .await;
        let config_id = restore_config(&context).await;
        let session_id = stopped_session(
            &context,
            config_id,
            REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED,
        )
        .await;

        assert!(
            load_persisted_startup_restore(&context, true)
                .await
                .is_none()
        );
        context.app_config.runtime.restore_connection_on_start = true;
        let restore = load_persisted_startup_restore(&context, false)
            .await
            .expect("enabled graceful shutdown should restore");
        assert_eq!(restore.config_id, config_id);
        assert_eq!(restore.source_session_id, session_id);
        assert_eq!(restore.kind, StartupRestoreKind::GracefulShutdown);

        context
            .db
            .update_runtime_session_transition_metadata(
                session_id,
                Some("cli"),
                None,
                Some(REASON_MANUAL_DISCONNECT),
                None,
                Some("cli"),
            )
            .await
            .unwrap();
        assert!(
            load_persisted_startup_restore(&context, true)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn stale_pid_restore_respects_daemon_recovery_policy() {
        let (context, _root) =
            crate::app::tests::TestAppBuilder::new("startup-restore-stale-policy")
                .build_with_root()
                .await;
        let config_id = restore_config(&context).await;
        stopped_session(&context, config_id, REASON_DAEMON_STALE_PID_RESTORE_PENDING).await;

        assert!(
            load_persisted_startup_restore(&context, false)
                .await
                .is_none()
        );
        assert_eq!(
            load_persisted_startup_restore(&context, true)
                .await
                .expect("enabled stale-PID recovery should restore")
                .kind,
            StartupRestoreKind::StalePid
        );
    }

    #[tokio::test]
    async fn failed_restore_persists_intent_and_schedules_bounded_backoff() {
        let (mut context, _root) = crate::app::tests::TestAppBuilder::new("startup-restore-retry")
            .build_with_root()
            .await;
        let config_id = restore_config(&context).await;
        let session_id = stopped_session(
            &context,
            config_id,
            REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED,
        )
        .await;
        context.app_config.runtime.restore_connection_on_start = true;
        let mut restore = load_persisted_startup_restore(&context, true)
            .await
            .expect("restore candidate should load");

        schedule_restore_retry(
            &context,
            "current-daemon",
            &mut restore,
            "network unavailable".to_string(),
        )
        .await;

        assert_eq!(
            restore.next_retry_seconds,
            STARTUP_RESTORE_INITIAL_RETRY_SECONDS * 2
        );
        assert!(restore.retry_at > time::Instant::now());
        let latest = context
            .db
            .get_latest_runtime_session()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(latest.id, session_id);
        assert_eq!(
            latest.last_transition_reason_code.as_deref(),
            Some(REASON_DAEMON_STARTUP_RESTORE_PENDING)
        );
        assert_eq!(
            load_persisted_startup_restore(&context, true)
                .await
                .expect("retry intent should survive daemon restart")
                .config_id,
            config_id
        );
    }

    #[test]
    fn startup_restore_retry_backoff_is_exponential_and_capped() {
        assert_eq!(next_startup_restore_retry_delay_secs(5), 10);
        assert_eq!(next_startup_restore_retry_delay_secs(160), 300);
        assert_eq!(next_startup_restore_retry_delay_secs(300), 300);
    }

    #[tokio::test]
    async fn shutdown_exits_supervisor_before_later_queued_requests() {
        let (context, _root) = crate::app::tests::TestAppBuilder::new("supervisor-shutdown")
            .build_with_root()
            .await;
        let (sender, receiver) = channel(4);
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let (ping_tx, ping_rx) = tokio::sync::oneshot::channel();
        sender
            .send(SupervisorEvent::DaemonShutdown {
                respond_to: shutdown_tx,
            })
            .await
            .unwrap();
        sender
            .send(SupervisorEvent::DaemonPing {
                respond_to: ping_tx,
            })
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(5), run(receiver, context))
            .await
            .unwrap();
        assert!(matches!(
            shutdown_rx.await.unwrap(),
            DaemonShutdownResult::Ok(_)
        ));
        assert!(ping_rx.await.is_err());
    }
}
