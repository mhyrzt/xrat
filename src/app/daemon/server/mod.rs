use std::io::ErrorKind;
use std::path::{Path, PathBuf};

mod bridge;
mod client;
mod responses;
mod serve;
mod types;

pub use client::{
    daemon_shutdown_daemon, ping_daemon, proxy_start_daemon, proxy_status_daemon,
    proxy_stop_daemon, runtime_connect_daemon, runtime_disconnect_daemon, runtime_replace_daemon,
    runtime_status_daemon,
};
pub use responses::{
    daemon_shutdown_response, ping_response, proxy_control_error_response, proxy_control_response,
    proxy_status_error_response, proxy_status_response, runtime_connect_error_response,
    runtime_connect_response, runtime_disconnect_error_response, runtime_disconnect_response,
    runtime_replace_error_response, runtime_replace_response, runtime_status_error_response,
    runtime_status_response,
};
pub use serve::serve_ping;
pub use types::*;

pub const PROTOCOL_VERSION: u16 = 1;

pub fn default_socket_path(runtime_dir: &Path) -> PathBuf {
    runtime_dir.join("daemon.sock")
}

pub fn daemon_unreachable(err: &crate::app::AppError) -> bool {
    match err {
        crate::app::AppError::Io(io_err) => matches!(
            io_err.kind(),
            ErrorKind::NotFound | ErrorKind::ConnectionRefused | ErrorKind::ConnectionReset
        ),
        _ => false,
    }
}

#[cfg(all(test, unix))]
mod tests;
