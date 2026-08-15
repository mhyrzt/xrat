mod config;
pub mod process_mgmt;

pub use config::{
    SingboxClashApi, SingboxConfig, SingboxInbound, SingboxInboundUser, SingboxRouteList,
    SingboxRoutingOptions, generate_singbox_probe_config, generate_singbox_runtime_config,
};
pub use process_mgmt::{
    ManagedSingboxPaths, ManagedSingboxProcess, SingboxRuntimeError, spawn_detached,
};
