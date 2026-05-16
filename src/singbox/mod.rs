mod config;
mod process_mgmt;

pub use config::{SingboxConfig, generate_parse_config};
pub use process_mgmt::{
    ManagedSingboxPaths, ManagedSingboxProcess, SingboxRuntimeError, spawn_detached,
};
