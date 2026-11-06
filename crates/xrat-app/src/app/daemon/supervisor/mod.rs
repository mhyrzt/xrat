use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::mpsc;
use tokio::time::{self, Duration};

mod handlers;
mod startup_restore;
mod types;

use startup_restore::{
    attempt_startup_restore, load_persisted_startup_restore, load_stale_pid_restore,
};

pub use types::{
    DaemonShutdownResult, ProxyControlResult, ProxyStatusResult, RuntimeConnectResult,
    RuntimeDisconnectResult, RuntimeReplaceResult, RuntimeStatusResult, SupervisorEvent,
    SupervisorState, channel,
};

use crate::app::context::AppContext;
use crate::app::events;
use crate::app::runtime_service::{
    ConnectRequest, REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED,
    REASON_DAEMON_STALE_PID_RESTORE_PENDING, REASON_DAEMON_STARTUP_RESTORE_PENDING,
    REASON_DAEMON_STARTUP_RESTORE_UNAVAILABLE, RuntimeService,
};
use crate::app::subscription_refresh;
use xrat_model::ConfigId;
use xrat_support::time::now_epoch_seconds;

mod state;
pub use state::run;
#[cfg(windows)]
pub use state::run_machine;
