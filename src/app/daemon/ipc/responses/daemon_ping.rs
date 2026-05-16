use super::ok_response;
use crate::app::daemon::ipc::{DaemonResponse, DaemonShutdownPayload, PingPayload};

pub fn ping_response() -> DaemonResponse<PingPayload> {
    ok_response("daemon reachable", PingPayload { daemon_ready: true })
}

pub fn daemon_shutdown_response(
    payload: DaemonShutdownPayload,
) -> DaemonResponse<DaemonShutdownPayload> {
    ok_response("daemon shutdown requested", payload)
}
