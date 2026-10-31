#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod connect;
mod helpers;
mod killswitch;
mod launch;
pub(crate) mod log_retention;
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
use spawn::*;
use types::*;
pub use types::{
    ConnectRequest, ConnectResult, DisconnectResult, ReplaceRequest, ReplaceResult,
    RuntimeEndpoint, RuntimeEndpointHealth, RuntimeEndpointState, RuntimeEndpoints,
    RuntimeInboundHealth, RuntimeService, RuntimeSessionDisplay, RuntimeStatusSnapshot,
};

#[cfg(test)]
mod tests;
