mod config;
mod runtime;

pub use config::{SingboxConfig, generate_parse_config};
pub use runtime::{
    ManagedSingboxPaths, ManagedSingboxProcess, SingboxRuntimeError, spawn_detached,
};
