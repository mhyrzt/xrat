use crate::app::daemon::server::{
    DaemonShutdownPayload, PingPayload, RuntimeConnectPayload, RuntimeDisconnectPayload,
    RuntimeReplacePayload, RuntimeStatusPayload,
};
use crate::app::daemon::supervisor::{
    DaemonShutdownResult, RuntimeConnectResult, RuntimeDisconnectResult, RuntimeReplaceResult,
    RuntimeStatusResult, SupervisorEvent, SupervisorState,
};
use crate::app::runtime::AppContext;
use crate::app::runtime_service::{ConnectRequest, ReplaceRequest, RuntimeService};
use std::time::{SystemTime, UNIX_EPOCH};

const HEALTH_FAILURE_COOLDOWN_SECONDS: u64 = 300;

pub async fn handle_event(
    state: &mut SupervisorState,
    event: SupervisorEvent,
    context: &AppContext,
) {
    match event {
        SupervisorEvent::HealthTick => {
            if let Ok(snapshot) = RuntimeService::new(context).status().await {
                if let Some(session) = snapshot.session {
                    if snapshot.pid_running && snapshot.inbound_health.has_unreachable_endpoint() {
                        if !should_record_health_failure(&session) {
                            return;
                        }
                        let failed_at = now_epoch_seconds();
                        let cooldown_until = (failed_at + HEALTH_FAILURE_COOLDOWN_SECONDS).to_string();
                        let failed_at = failed_at.to_string();
                        let _ = context
                            .db
                            .update_runtime_session_transition_metadata(
                                session.id,
                                Some("daemon"),
                                Some(&state.instance_id),
                                Some("health_check_failed"),
                                Some(
                                    "runtime health check detected unreachable inbound endpoint",
                                ),
                                Some("daemon"),
                            )
                            .await;
                        let _ = context
                            .db
                            .update_runtime_session_failure_tracking(
                                session.id,
                                Some(&cooldown_until),
                                Some(&failed_at),
                                Some("health_check_failed"),
                            )
                            .await;
                    }
                }
            }
        }
        SupervisorEvent::DaemonPing { respond_to } => {
            state.ready = true;
            let _ = respond_to.send(PingPayload {
                daemon_ready: state.ready,
            });
        }
        SupervisorEvent::RuntimeStatus { respond_to } => {
            match RuntimeService::new(context).status().await {
                Ok(snapshot) => {
                    let _ = respond_to.send(RuntimeStatusResult::Ok(RuntimeStatusPayload {
                        daemon_ready: state.ready,
                        runtime_owned: snapshot.session.is_some() && snapshot.pid_running,
                        runtime_status: snapshot.status.as_str().to_string(),
                        session_id: snapshot.session.as_ref().map(|session| session.id),
                        active_config_id: snapshot.active_config.as_ref().map(|config| config.id),
                        pid_running: snapshot.pid_running,
                    }));
                }
                Err(err) => {
                    let _ = respond_to.send(RuntimeStatusResult::Err {
                        message: err.to_string(),
                    });
                }
            }
        }
        SupervisorEvent::RuntimeConnect {
            config_id,
            respond_to,
        } => {
            match RuntimeService::new(context)
                .connect(ConnectRequest { config_id })
                .await
            {
                Ok(result) => {
                    let _ = context
                        .db
                        .update_runtime_session_transition_metadata(
                            result.session_id,
                            Some("daemon"),
                            Some(&state.instance_id),
                            Some("manual_connect"),
                            Some("daemon runtime connect request succeeded"),
                            Some("daemon"),
                        )
                        .await;
                    let _ = respond_to.send(RuntimeConnectResult::Ok(RuntimeConnectPayload {
                        config_id: result.config.id,
                        session_id: result.session_id,
                        pid: result.pid,
                    }));
                }
                Err(err) => {
                    let _ = respond_to.send(RuntimeConnectResult::Err {
                        message: err.to_string(),
                    });
                }
            }
        }
        SupervisorEvent::RuntimeDisconnect { respond_to } => {
            let active_session_id = context
                .db
                .get_running_runtime_session()
                .await
                .ok()
                .flatten()
                .map(|session| session.id);
            match RuntimeService::new(context).disconnect().await {
                Ok(result) => {
                    if result.stopped_session {
                        if let Some(session_id) = active_session_id {
                            let _ = context
                                .db
                                .update_runtime_session_transition_metadata(
                                    session_id,
                                    Some("daemon"),
                                    Some(&state.instance_id),
                                    Some("manual_disconnect"),
                                    Some("daemon runtime disconnect request succeeded"),
                                    Some("daemon"),
                                )
                                .await;
                        }
                    }
                    let _ =
                        respond_to.send(RuntimeDisconnectResult::Ok(RuntimeDisconnectPayload {
                            stopped_session: result.stopped_session,
                        }));
                }
                Err(err) => {
                    let _ = respond_to.send(RuntimeDisconnectResult::Err {
                        message: err.to_string(),
                    });
                }
            }
        }
        SupervisorEvent::RuntimeReplace {
            trigger,
            candidate_id,
            respond_to,
        } => {
            match RuntimeService::new(context)
                .replace(ReplaceRequest {
                    trigger,
                    candidate_id,
                })
                .await
            {
                Ok(result) => {
                    let _ = context
                        .db
                        .update_runtime_session_transition_metadata(
                            result.new_session_id,
                            Some("daemon"),
                            Some(&state.instance_id),
                            Some("replace_commit_success"),
                            Some("daemon replace handoff completed"),
                            Some("daemon"),
                        )
                        .await;
                    let _ = respond_to.send(RuntimeReplaceResult::Ok(RuntimeReplacePayload {
                        trigger,
                        replaced: true,
                        old_session_id: result.old_session_id,
                        new_config_id: result.new_config_id,
                        new_session_id: result.new_session_id,
                        new_pid: result.new_pid,
                    }));
                }
                Err(err) => {
                    let _ = respond_to.send(RuntimeReplaceResult::Err {
                        message: err.to_string(),
                    });
                }
            }
        }
        SupervisorEvent::DaemonShutdown { respond_to } => {
            let runtime_disconnected = RuntimeService::new(context)
                .disconnect()
                .await
                .map(|result| result.stopped_session)
                .unwrap_or(false);
            let _ = respond_to.send(DaemonShutdownResult::Ok(DaemonShutdownPayload {
                daemon_ready: false,
                runtime_disconnected,
            }));
        }
    }
}

fn now_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn should_record_health_failure(session: &crate::db::RuntimeSessionRecord) -> bool {
    if session.last_failed_reason_code.as_deref() != Some("health_check_failed") {
        return true;
    }
    let Some(cooldown_until) = session.cooldown_until.as_deref() else {
        return true;
    };
    let Ok(cooldown_until) = cooldown_until.parse::<u64>() else {
        return true;
    };
    now_epoch_seconds() >= cooldown_until
}
