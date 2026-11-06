#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod connect;
mod helpers;
mod killswitch;
mod launch;
pub(crate) mod log_retention;
mod raw_profile;
mod reattach;
mod replace_flow;
mod rollback;
mod session_state;
mod spawn;
mod status;
mod tun_interface;
pub(crate) mod tun_ownership;
mod types;

use helpers::*;
use session_state::*;
pub(crate) use session_state::{
    REASON_DAEMON_SHUTDOWN_RESTORE_REQUESTED, REASON_DAEMON_STALE_PID_RESTORE_PENDING,
    REASON_DAEMON_STARTUP_RESTORE_PENDING, REASON_DAEMON_STARTUP_RESTORE_SUCCEEDED,
    REASON_DAEMON_STARTUP_RESTORE_UNAVAILABLE, REASON_MANUAL_CONNECT, REASON_MANUAL_DISCONNECT,
    StopTransition, cancel_pending_startup_restore, stop_active_session_with_transition,
};
use spawn::*;
use types::*;
pub use types::{
    ConnectRequest, ConnectResult, DisconnectResult, ReplaceRequest, ReplaceResult,
    RuntimeEndpoint, RuntimeEndpointHealth, RuntimeEndpointState, RuntimeEndpoints,
    RuntimeInboundHealth, RuntimeService, RuntimeSessionDisplay, RuntimeStatusSnapshot,
};

#[cfg(test)]
mod tests;
