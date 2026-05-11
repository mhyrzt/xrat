use tokio::sync::{mpsc, oneshot};

use crate::app::daemon::server::{
    DaemonShutdownPayload, PingPayload, RotationTrigger, RuntimeConnectPayload,
    RuntimeDisconnectPayload, RuntimeReplacePayload, RuntimeStatusPayload,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisorState {
    pub ready: bool,
    pub instance_id: String,
}

impl SupervisorState {
    pub fn new(instance_id: String) -> Self {
        Self {
            ready: true,
            instance_id,
        }
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
