use tokio::sync::mpsc;

mod handlers;
mod types;

pub use types::{
    DaemonShutdownResult, RuntimeConnectResult, RuntimeDisconnectResult, RuntimeReplaceResult,
    RuntimeStatusResult, SupervisorEvent, SupervisorState, channel,
};

use crate::app::runtime::AppContext;
use crate::app::runtime_service::RuntimeService;

pub async fn run(mut rx: mpsc::Receiver<SupervisorEvent>, context: AppContext) {
    if let Err(err) = RuntimeService::new(&context)
        .reconcile_reattach_on_daemon_start()
        .await
    {
        tracing::warn!(error = %err, "daemon reattach reconciliation failed");
    }
    let mut state = SupervisorState::default();
    while let Some(event) = rx.recv().await {
        handlers::handle_event(&mut state, event, &context).await;
    }
}
