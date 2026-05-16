use tokio::sync::mpsc;
use tokio::time::{self, Duration};

mod handlers;
mod types;

pub use types::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeConnectResult,
    RuntimeDisconnectResult, RuntimeReplaceResult, RuntimeStatusResult, SupervisorEvent,
    SupervisorState, channel,
};

use crate::app::runtime::AppContext;
use crate::app::runtime_service::RuntimeService;
use crate::support::time::now_epoch_seconds;

const HEALTH_TICK_SECONDS: u64 = 15;

pub async fn run(mut rx: mpsc::Receiver<SupervisorEvent>, context: AppContext) {
    let daemon_instance_id = uuid::Uuid::new_v4().to_string();
    if let Err(err) = RuntimeService::new(&context)
        .reconcile_reattach_on_daemon_start(&daemon_instance_id)
        .await
    {
        tracing::warn!(error = %err, "daemon reattach reconciliation failed");
    }
    let mut state = SupervisorState::new(daemon_instance_id);
    state.rotation_enabled = context.app_config.runtime.rotation.enabled;
    state.rotation_interval_secs = context.app_config.runtime.rotation.interval_secs;
    state.health_trigger_enabled = context.app_config.runtime.rotation.health_trigger_enabled;
    state.cooldown_secs = context.app_config.runtime.rotation.cooldown_secs;
    if state.rotation_enabled {
        state.next_timer_epoch_secs = Some(now_epoch_seconds() + state.rotation_interval_secs);
    }
    let mut health_ticker = time::interval(Duration::from_secs(HEALTH_TICK_SECONDS));
    health_ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = health_ticker.tick() => {
                handlers::handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;
            }
            event = rx.recv() => {
                let Some(event) = event else {
                    break;
                };
                handlers::handle_event(&mut state, event, &context).await;
            }
        }
    }
}
