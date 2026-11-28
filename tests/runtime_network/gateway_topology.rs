#[path = "gateway_topology/forwarding.rs"]
mod forwarding;
#[path = "gateway_topology/peer.rs"]
mod peer;
#[path = "gateway_topology/workers.rs"]
mod workers;

pub use forwarding::exercise;
use peer::Peer;
pub use workers::{client_worker, closed_client_worker, echo_worker};

#[path = "gateway_topology/managed_engines.rs"]
mod managed_engines;
pub use managed_engines::exercise_managed_engines;

use std::{num::NonZeroU32, process::Command};

use xrat_support::net::firewall::gateway;

use super::{
    fixtures,
    process::{self, ChildGuard},
};

const CAPTURE_ECHO: &str = "198.51.100.2";
