use super::{error_response, ok_response};
use crate::app::daemon::server::{
    DaemonResponse, DaemonResponseCode, ProxyControlPayload, ProxyStatusPayload,
};

pub fn proxy_control_response(
    payload: ProxyControlPayload,
    message: &str,
) -> DaemonResponse<ProxyControlPayload> {
    ok_response(message, payload)
}

pub fn proxy_control_error_response(message: String) -> DaemonResponse<ProxyControlPayload> {
    error_response(DaemonResponseCode::InvalidState, message)
}

pub fn proxy_status_response(payload: ProxyStatusPayload) -> DaemonResponse<ProxyStatusPayload> {
    ok_response("proxy rotation status available", payload)
}

pub fn proxy_status_error_response(message: String) -> DaemonResponse<ProxyStatusPayload> {
    error_response(DaemonResponseCode::InternalError, message)
}
