pub mod config;
pub mod parsing;
pub mod process;
pub mod process_mgmt;

pub use config::{
    XrayConfig, generate_probe_config, generate_runtime_config,
    generate_runtime_config_for_inbounds, generate_runtime_config_with_inbounds,
};
pub use parsing::{ParseMode, XrayConfig as XrayConfigJson, XrayConfigError};
pub use process::{XrayProcess, XrayProcessError};
