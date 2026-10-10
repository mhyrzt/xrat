mod config;
mod probe;
pub mod process_mgmt;
mod validation;
mod version;
pub use validation::config_check_command;

pub use config::{
    SingboxCacheFile, SingboxClashApi, SingboxConfig, SingboxDnsConfig, SingboxExperimental,
    SingboxInbound, SingboxInboundUser, SingboxLogConfig, SingboxRoute, SingboxRouteList,
    SingboxRoutingOptions, SingboxTunOptions, generate_singbox_probe_config,
    generate_singbox_runtime_config, generate_singbox_runtime_config_with_dns,
};
pub use probe::{SingboxProbeError, SingboxProbeProcess};
pub use process_mgmt::{
    ManagedSingboxPaths, ManagedSingboxProcess, SingboxRuntimeError, spawn_detached,
};
pub use version::SingboxVersionError;
pub use version::{ensure_supported_binary, ensure_supported_binary_with_spawner};
