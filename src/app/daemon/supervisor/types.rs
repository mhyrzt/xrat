use tokio::sync::{mpsc, oneshot};

use crate::app::daemon::server::{
    DaemonShutdownPayload, PingPayload, ProxyControlPayload, ProxyStatusPayload, RotationTrigger,
    RuntimeConnectPayload, RuntimeDisconnectPayload, RuntimeReplacePayload, RuntimeStatusPayload,
};

#[derive(Debug)]
pub enum SupervisorEvent {
    HealthTick,
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
    ProxyStart {
        respond_to: oneshot::Sender<ProxyControlResult>,
    },
    ProxyStatus {
        respond_to: oneshot::Sender<ProxyStatusResult>,
    },
    ProxyStop {
        respond_to: oneshot::Sender<ProxyControlResult>,
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

#[derive(Debug)]
pub enum ProxyControlResult {
    Ok(ProxyControlPayload),
    Err { message: String },
}

#[derive(Debug)]
pub enum ProxyStatusResult {
    Ok(ProxyStatusPayload),
    Err { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupervisorState {
    pub ready: bool,
    pub instance_id: String,
    pub rotation_enabled: bool,
    pub rotation_interval_secs: u64,
    pub health_trigger_enabled: bool,
    pub cooldown_secs: u64,
    pub next_timer_epoch_secs: Option<u64>,
    pub last_trigger: Option<RotationTrigger>,
    pub last_result: String,
    pub last_candidate_config_id: Option<i64>,
    pub last_candidate_result: String,
    pub cooldown_active: bool,
}

impl SupervisorState {
    pub fn new(instance_id: String) -> Self {
        Self {
            ready: true,
            instance_id,
            rotation_enabled: false,
            rotation_interval_secs: 1800,
            health_trigger_enabled: true,
            cooldown_secs: 300,
            next_timer_epoch_secs: None,
            last_trigger: None,
            last_result: "never_triggered".to_string(),
            last_candidate_config_id: None,
            last_candidate_result: "never_selected".to_string(),
            cooldown_active: false,
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
