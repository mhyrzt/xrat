mod config;
mod process_mgmt;

pub use config::{SingboxConfig, generate_singbox_probe_config};
pub use process_mgmt::{
    ManagedSingboxPaths, ManagedSingboxProcess, SingboxRuntimeError, spawn_detached,
};
