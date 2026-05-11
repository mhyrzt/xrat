use super::*;

mod inbound_health;
mod lifecycle;

pub(crate) use inbound_health::{check_runtime_inbounds, runtime_status_label};
pub(crate) use lifecycle::{
    active_session_state, runtime_session_is_alive, stop_active_session, stop_session,
};

pub(super) struct ResolvedLaunch {
    pub(super) binary_path: PathBuf,
    pub(super) config: crate::xray::XrayConfig,
    pub(super) ready_host: String,
    pub(super) ready_port: u16,
    pub(super) endpoints: RuntimeEndpoints,
}
