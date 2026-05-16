use super::{error_response, ok_response};
use crate::app::daemon::ipc::{
    DaemonResponse, DaemonResponseCode, RuntimeConnectPayload, RuntimeDisconnectPayload,
    RuntimeReplacePayload, RuntimeStatusPayload,
};

pub fn runtime_status_response(
    payload: RuntimeStatusPayload,
) -> DaemonResponse<RuntimeStatusPayload> {
    ok_response("runtime status available", payload)
}

pub fn runtime_status_error_response(message: String) -> DaemonResponse<RuntimeStatusPayload> {
    error_response(DaemonResponseCode::InternalError, message)
}

pub fn runtime_connect_response(
    payload: RuntimeConnectPayload,
) -> DaemonResponse<RuntimeConnectPayload> {
    ok_response("runtime connected", payload)
}

pub fn runtime_connect_error_response(message: String) -> DaemonResponse<RuntimeConnectPayload> {
    error_response(DaemonResponseCode::InvalidState, message)
}

pub fn runtime_disconnect_response(
    payload: RuntimeDisconnectPayload,
) -> DaemonResponse<RuntimeDisconnectPayload> {
    ok_response("runtime disconnected", payload)
}

pub fn runtime_disconnect_error_response(
    message: String,
) -> DaemonResponse<RuntimeDisconnectPayload> {
    error_response(DaemonResponseCode::InvalidState, message)
}

pub fn runtime_replace_response(
    payload: RuntimeReplacePayload,
) -> DaemonResponse<RuntimeReplacePayload> {
    ok_response("runtime replaced", payload)
}

pub fn runtime_replace_error_response(message: String) -> DaemonResponse<RuntimeReplacePayload> {
    error_response(DaemonResponseCode::InvalidState, message)
}
