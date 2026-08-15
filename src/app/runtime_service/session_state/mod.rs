use super::*;

mod inbound_health;
mod lifecycle;

pub(crate) use inbound_health::{check_runtime_inbounds, runtime_status_label};
pub(crate) use lifecycle::{
    active_session_state, runtime_session_is_alive, stop_active_session, stop_session,
};

pub(super) struct ResolvedLaunch {
    pub(super) binary_path: PathBuf,
    pub(super) config: RuntimeLaunchConfig,
    pub(super) ready_host: String,
    pub(super) ready_port: u16,
    pub(super) endpoints: RuntimeEndpoints,
    pub(super) validator: RuntimeValidator,
}

pub(super) enum RuntimeLaunchConfig {
    Xray(crate::xray::XrayConfig),
    Singbox(SingboxConfig),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RuntimeValidator {
    Xray,
    V2ray,
    Singbox,
}
