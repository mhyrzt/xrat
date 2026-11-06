use super::*;

#[cfg(test)]
pub async fn handle_event(
    state: &mut SupervisorState,
    event: SupervisorEvent,
    context: &AppContext,
) {
    let mut context = context.clone();
    handle_event_inner(state, event, &mut context, None).await;
}

pub(in super::super) async fn handle_event_with_sender(
    state: &mut SupervisorState,
    event: SupervisorEvent,
    context: &mut AppContext,
    event_tx: &tokio::sync::mpsc::Sender<SupervisorEvent>,
) {
    handle_event_inner(state, event, context, Some(event_tx)).await;
}

pub(super) async fn handle_event_inner(
    state: &mut SupervisorState,
    event: SupervisorEvent,
    context: &mut AppContext,
    event_tx: Option<&tokio::sync::mpsc::Sender<SupervisorEvent>>,
) {
    match event {
        SupervisorEvent::HealthTick => {
            let outcome = health::handle_health_tick(state, context).await;
            state.cooldown_active = outcome.cooldown_active;
            if state.rotation_enabled
                && state.health_trigger_enabled
                && outcome.health_failure_recorded
            {
                let (tx, _rx) = oneshot::channel();
                runtime::handle_runtime_replace(
                    state,
                    context,
                    RotationTrigger::HealthCheckFailed,
                    None,
                    tx,
                )
                .await;
            } else if state.rotation_enabled && outcome.timer_due {
                let (tx, _rx) = oneshot::channel();
                runtime::handle_runtime_replace(state, context, RotationTrigger::Timer, None, tx)
                    .await;
            }
            if let Some(probe) = outcome.probe {
                if let Some(event_tx) = event_tx {
                    let event_tx = event_tx.clone();
                    tokio::spawn(async move {
                        let (session_id, success, error) = health::execute_probe(probe).await;
                        if event_tx
                            .send(SupervisorEvent::HealthProbeCompleted {
                                session_id,
                                success,
                                error,
                            })
                            .await
                            .is_err()
                        {
                            tracing::debug!(
                                operation = "health_probe_completed_send",
                                session_id,
                                error = "supervisor event receiver dropped",
                                "health probe result dropped"
                            );
                        }
                    });
                } else {
                    state.health_probe_in_flight = false;
                }
            }
        }
        SupervisorEvent::HealthProbeCompleted {
            session_id,
            success,
            error,
        } => {
            if health::handle_probe_completed(state, context, session_id, success, error).await
                && state.rotation_enabled
            {
                let (tx, _rx) = oneshot::channel();
                runtime::handle_runtime_replace(
                    state,
                    context,
                    RotationTrigger::HealthCheckFailed,
                    None,
                    tx,
                )
                .await;
            }
        }
        SupervisorEvent::DaemonPing { respond_to } => {
            state.ready = true;
            if respond_to
                .send(PingPayload {
                    daemon_ready: state.ready,
                    live_tun: true,
                })
                .is_err()
            {
                tracing::debug!(
                    operation = "daemon_ping_response",
                    error = "response receiver dropped",
                    "supervisor response dropped"
                );
            }
        }
        SupervisorEvent::RuntimeTun {
            enabled,
            config_path,
            respond_to,
        } => {
            let same_config = std::fs::canonicalize(&config_path)
                .ok()
                .zip(std::fs::canonicalize(&context.runtime_paths.config_path).ok())
                .is_some_and(|(requested, owned)| requested == owned);
            let result = if same_config {
                crate::app::services::tun::apply(context, enabled)
                    .await
                    .map_err(|error| error.to_string())
            } else {
                Err("The daemon uses a different config file. Use its config path with `xrat --config <path> tun ...`.".to_string())
            };
            if same_config
                && let Ok(snapshot) = crate::app::runtime_service::RuntimeService::new(context)
                    .status()
                    .await
                && let Some(session) = snapshot.session.filter(|session| {
                    snapshot.pid_running && session.owner_kind.as_deref() != Some("daemon")
                })
                && let Err(error) = context
                    .db
                    .update_runtime_session_transition_metadata(
                        session.id,
                        Some("daemon"),
                        Some(&state.instance_id),
                        Some(if result.is_ok() {
                            "tun_mode_changed"
                        } else {
                            "tun_mode_rollback"
                        }),
                        Some(if result.is_ok() {
                            "daemon TUN mode change completed"
                        } else {
                            "daemon restored the runtime after a TUN failure"
                        }),
                        Some("daemon"),
                    )
                    .await
            {
                tracing::warn!(%error, "TUN runtime owner metadata update failed");
            }
            let _ = respond_to.send(result);
        }
        SupervisorEvent::RuntimeOverview { respond_to } => {
            runtime::handle_runtime_overview(state, context, respond_to).await;
        }
        SupervisorEvent::RuntimeTraffic { respond_to } => {
            runtime::handle_runtime_traffic(context, respond_to).await;
        }
        SupervisorEvent::RuntimeDashboard { respond_to } => {
            runtime::handle_runtime_dashboard(context, respond_to).await;
        }
        SupervisorEvent::RuntimeStatus { respond_to } => {
            runtime::handle_runtime_status(state, context, respond_to).await;
        }
        SupervisorEvent::RuntimeConnect {
            config_id,
            respond_to,
        } => {
            runtime::handle_runtime_connect(state, context, config_id, respond_to).await;
        }
        SupervisorEvent::RuntimeDisconnect { respond_to } => {
            runtime::handle_runtime_disconnect(state, context, respond_to).await;
        }
        SupervisorEvent::RuntimeReplace {
            trigger,
            candidate_id,
            respond_to,
        } => {
            runtime::handle_runtime_replace(state, context, trigger, candidate_id, respond_to)
                .await;
        }
        SupervisorEvent::DaemonShutdown { respond_to } => {
            runtime::handle_daemon_shutdown(state, context, respond_to).await;
        }
        SupervisorEvent::ProxyStart { respond_to } => {
            runtime::handle_proxy_start(state, context, respond_to).await;
        }
        SupervisorEvent::ProxyStatus { respond_to } => {
            runtime::handle_proxy_status(state, context, respond_to).await;
        }
        SupervisorEvent::ProxyStop { respond_to } => {
            runtime::handle_proxy_stop(state, context, respond_to).await;
        }
    }
}
