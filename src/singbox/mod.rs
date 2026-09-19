mod config;
mod probe;
pub mod process_mgmt;
mod version;

pub use config::{
    SingboxCacheFile, SingboxClashApi, SingboxConfig, SingboxDnsConfig, SingboxExperimental,
    SingboxInbound, SingboxInboundUser, SingboxRouteList, SingboxRoutingOptions,
    generate_singbox_probe_config, generate_singbox_runtime_config,
    generate_singbox_runtime_config_with_dns,
};
pub use probe::{SingboxProbeError, SingboxProbeProcess};
pub use process_mgmt::{
    ManagedSingboxPaths, ManagedSingboxProcess, SingboxRuntimeError, spawn_detached,
};
pub(crate) use version::ensure_supported_binary;
