mod config;
pub mod process_mgmt;

pub use config::{
    SingboxClashApi, SingboxConfig, SingboxDnsConfig, SingboxInbound, SingboxInboundUser,
    SingboxRouteList, SingboxRoutingOptions, generate_singbox_probe_config,
    generate_singbox_runtime_config, generate_singbox_runtime_config_with_dns,
};
pub use process_mgmt::{
    ManagedSingboxPaths, ManagedSingboxProcess, SingboxRuntimeError, spawn_detached,
};
