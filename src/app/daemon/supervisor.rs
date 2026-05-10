use tokio::sync::{mpsc, oneshot};

use crate::app::daemon::server::{
    DaemonShutdownPayload, PingPayload, RotationTrigger, RuntimeConnectPayload,
    RuntimeDisconnectPayload, RuntimeReplacePayload, RuntimeStatusPayload,
};
use crate::app::runtime::AppContext;
use crate::app::runtime_service::{ConnectRequest, ReplaceRequest, RuntimeService};

#[derive(Debug)]
pub enum SupervisorEvent {
    DaemonPing {
        respond_to: oneshot::Sender<PingPayload>,
    },
    RuntimeStatus {
        respond_to: oneshot::Sender<RuntimeStatusResult>,
    },
    RuntimeConnect {
        config_id: i64,
        respond_to: oneshot::Sender<RuntimeConnectResult>,
    },
    RuntimeDisconnect {
        respond_to: oneshot::Sender<RuntimeDisconnectResult>,
    },
    RuntimeReplace {
        trigger: RotationTrigger,
        candidate_id: Option<i64>,
        respond_to: oneshot::Sender<RuntimeReplaceResult>,
    },
    DaemonShutdown {
        respond_to: oneshot::Sender<DaemonShutdownResult>,
    },
}

#[derive(Debug)]
pub enum RuntimeConnectResult {
    Ok(RuntimeConnectPayload),
    Err { message: String },
}

#[derive(Debug)]
pub enum RuntimeStatusResult {
    Ok(RuntimeStatusPayload),
    Err { message: String },
}

#[derive(Debug)]
pub enum RuntimeDisconnectResult {
    Ok(RuntimeDisconnectPayload),
    Err { message: String },
}

#[derive(Debug)]
pub enum RuntimeReplaceResult {
    Ok(RuntimeReplacePayload),
    Err { message: String },
}

#[derive(Debug)]
pub enum DaemonShutdownResult {
    Ok(DaemonShutdownPayload),
    Err { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupervisorState {
    pub ready: bool,
}

impl Default for SupervisorState {
    fn default() -> Self {
        Self { ready: true }
    }
}

pub fn channel(
    buffer: usize,
) -> (
    mpsc::Sender<SupervisorEvent>,
    mpsc::Receiver<SupervisorEvent>,
) {
    mpsc::channel(buffer)
}

pub async fn run(mut rx: mpsc::Receiver<SupervisorEvent>, context: AppContext) {
    if let Err(err) = RuntimeService::new(&context)
        .reconcile_reattach_on_daemon_start()
        .await
    {
        tracing::warn!(error = %err, "daemon reattach reconciliation failed");
    }
    let mut state = SupervisorState::default();
    while let Some(event) = rx.recv().await {
        handle_event(&mut state, event, &context).await;
    }
}

async fn handle_event(state: &mut SupervisorState, event: SupervisorEvent, context: &AppContext) {
    match event {
        SupervisorEvent::DaemonPing { respond_to } => {
            state.ready = true;
            let _ = respond_to.send(PingPayload {
                daemon_ready: state.ready,
            });
        }
        SupervisorEvent::RuntimeStatus { respond_to } => {
            match RuntimeService::new(context).status().await {
                Ok(snapshot) => {
                    let runtime_owned = snapshot.session.is_some() && snapshot.pid_running;
                    let runtime_status = snapshot.status.as_str().to_string();
                    let session_id = snapshot.session.as_ref().map(|session| session.id);
                    let active_config_id = snapshot.active_config.as_ref().map(|config| config.id);
                    let pid_running = snapshot.pid_running;
                    let _ = respond_to.send(RuntimeStatusResult::Ok(RuntimeStatusPayload {
                        daemon_ready: state.ready,
                        runtime_owned,
                        runtime_status,
                        session_id,
                        active_config_id,
                        pid_running,
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
        } => match RuntimeService::new(context)
            .connect(ConnectRequest { config_id })
            .await
        {
            Ok(result) => {
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
        },
        SupervisorEvent::RuntimeDisconnect { respond_to } => {
            match RuntimeService::new(context).disconnect().await {
                Ok(result) => {
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
        } => match RuntimeService::new(context)
            .replace(ReplaceRequest {
                trigger: trigger.clone(),
                candidate_id,
            })
            .await
        {
            Ok(result) => {
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
        },
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
