use super::*;

pub(in super::super) struct ResolvedLaunch {
    pub(in super::super) binary_path: PathBuf,
    pub(in super::super) config: RuntimeLaunchConfig,
    pub(in super::super) ready_host: String,
    pub(in super::super) ready_port: u16,
    pub(in super::super) endpoints: RuntimeEndpoints,
    pub(in super::super) validator: RuntimeValidator,
    pub(in super::super) allowed_endpoints: Vec<std::net::SocketAddr>,
}

pub(in super::super) enum RuntimeLaunchConfig {
    Xray(xrat_engines::xray::XrayConfig),
    Singbox(SingboxConfig),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum RuntimeValidator {
    Xray,
    V2ray,
    Singbox,
}
