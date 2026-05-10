use crate::app::daemon::server::{
    DaemonResponse, DaemonResponseCode, DaemonShutdownPayload, PROTOCOL_VERSION, PingPayload,
    RuntimeConnectPayload, RuntimeDisconnectPayload, RuntimeReplacePayload, RuntimeStatusPayload,
};

pub fn ping_response() -> DaemonResponse<PingPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "daemon reachable".to_string(),
        payload: Some(PingPayload { daemon_ready: true }),
    }
}

pub fn runtime_status_response(
    payload: RuntimeStatusPayload,
) -> DaemonResponse<RuntimeStatusPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "runtime status available".to_string(),
        payload: Some(payload),
    }
}

pub fn runtime_status_error_response(message: String) -> DaemonResponse<RuntimeStatusPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        code: DaemonResponseCode::InternalError,
        message,
        payload: None,
    }
}

pub fn runtime_connect_response(
    payload: RuntimeConnectPayload,
) -> DaemonResponse<RuntimeConnectPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "runtime connected".to_string(),
        payload: Some(payload),
    }
}

pub fn runtime_connect_error_response(message: String) -> DaemonResponse<RuntimeConnectPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        code: DaemonResponseCode::InvalidState,
        message,
        payload: None,
    }
}

pub fn runtime_disconnect_response(
    payload: RuntimeDisconnectPayload,
) -> DaemonResponse<RuntimeDisconnectPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "runtime disconnected".to_string(),
        payload: Some(payload),
    }
}

pub fn runtime_disconnect_error_response(
    message: String,
) -> DaemonResponse<RuntimeDisconnectPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        code: DaemonResponseCode::InvalidState,
        message,
        payload: None,
    }
}

pub fn runtime_replace_response(
    payload: RuntimeReplacePayload,
) -> DaemonResponse<RuntimeReplacePayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "runtime replaced".to_string(),
        payload: Some(payload),
    }
}

pub fn runtime_replace_error_response(message: String) -> DaemonResponse<RuntimeReplacePayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: false,
        code: DaemonResponseCode::InvalidState,
        message,
        payload: None,
    }
}

pub fn daemon_shutdown_response(
    payload: DaemonShutdownPayload,
) -> DaemonResponse<DaemonShutdownPayload> {
    DaemonResponse {
        protocol_version: PROTOCOL_VERSION,
        ok: true,
        code: DaemonResponseCode::Ok,
        message: "daemon shutdown requested".to_string(),
        payload: Some(payload),
    }
}
