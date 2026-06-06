use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::mpsc;
use tokio::time::{self, Duration};

mod handlers;
mod types;

pub use types::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeConnectResult,
    RuntimeDisconnectResult, RuntimeReplaceResult, RuntimeStatusResult, SupervisorEvent,
    SupervisorState, channel,
};

use crate::app::context::AppContext;
use crate::app::events;
use crate::app::runtime_service::{ConnectRequest, RuntimeService};
use crate::app::subscription_refresh;
use crate::support::time::now_epoch_seconds;

const HEALTH_TICK_SECONDS: u64 = 15;
/// How often the daemon checks whether any URL-backed subscription is due for an
/// automatic refresh. The per-subscription cadence is governed by
/// `[subscriptions].refresh_interval_hours`; this only bounds detection lag.
const SUBSCRIPTION_REFRESH_TICK_SECONDS: u64 = 300;

pub async fn run(mut rx: mpsc::Receiver<SupervisorEvent>, context: AppContext) {
    let daemon_instance_id = uuid::Uuid::new_v4().to_string();
    match RuntimeService::new(&context)
        .reconcile_reattach_on_daemon_start(&daemon_instance_id)
        .await
    {
        Ok(Some(config_id)) => recover_runtime_after_stale_pid(&context, config_id).await,
        Ok(None) => {}
        Err(err) => tracing::warn!(error = %err, "daemon reattach reconciliation failed"),
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

    let auto_refresh_enabled = context.app_config.subscriptions.auto_refresh;
    let refresh_in_progress = Arc::new(AtomicBool::new(false));
    let mut refresh_ticker = time::interval(Duration::from_secs(SUBSCRIPTION_REFRESH_TICK_SECONDS));
    refresh_ticker.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = health_ticker.tick() => {
                handlers::handle_event(&mut state, SupervisorEvent::HealthTick, &context).await;
            }
            _ = refresh_ticker.tick(), if auto_refresh_enabled => {
                spawn_due_refresh(&context, &refresh_in_progress);
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

/// Relaunch the persisted runtime config after a stale-PID reattach rejection
/// (typically after a reboot left a dead proxy PID). Records a readable event
/// for either outcome so recovery is visible in `xrat logs`.
async fn recover_runtime_after_stale_pid(context: &AppContext, config_id: i64) {
    match RuntimeService::new(context)
        .connect(ConnectRequest { config_id })
        .await
    {
        Ok(result) => {
            events::record(
                &context.db,
                events::LEVEL_INFO,
                events::SOURCE_RUNTIME,
                "daemon_restart_stale_pid_recovered",
                format!("Reconnected config {config_id} after stale runtime PID on daemon start"),
                Some(config_id),
                Some(result.session_id),
                None,
            )
            .await;
        }
        Err(err) => {
            events::record(
                &context.db,
                events::LEVEL_WARN,
                events::SOURCE_RUNTIME,
                "daemon_restart_stale_pid_recovery_failed",
                format!("Failed to reconnect config {config_id} after stale runtime PID"),
                Some(config_id),
                None,
                Some(err.to_string()),
            )
            .await;
        }
    }
}

/// Spawn a detached task that refreshes any due URL-backed subscriptions, so a
/// slow provider fetch never stalls the supervisor loop. The guard ensures only
/// one refresh batch runs at a time even if ticks pile up.
fn spawn_due_refresh(context: &AppContext, refresh_in_progress: &Arc<AtomicBool>) {
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
