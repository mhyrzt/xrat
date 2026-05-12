use crate::app::daemon::server::PingPayload;
use crate::app::daemon::server::RotationTrigger;
use crate::app::daemon::supervisor::{SupervisorEvent, SupervisorState};
use crate::app::runtime::AppContext;
use tokio::sync::oneshot;

mod health;
mod runtime;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;

pub async fn handle_event(
    state: &mut SupervisorState,
    event: SupervisorEvent,
    context: &AppContext,
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
        }
        SupervisorEvent::DaemonPing { respond_to } => {
            state.ready = true;
            let _ = respond_to.send(PingPayload {
                daemon_ready: state.ready,
            });
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
            runtime::handle_daemon_shutdown(context, respond_to).await;
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
