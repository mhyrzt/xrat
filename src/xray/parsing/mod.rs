pub mod core;
pub mod protocols;
pub mod shared;
pub mod transports;

pub use core::*;
pub use protocols::*;
pub use shared::*;
pub use transports::*;

/// Parsing mode for Xray configuration
#[derive(Debug, Clone, Copy, Default, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParseMode {
    /// Strict mode: reject unknown fields
    #[default]
    Strict,
    /// Lenient mode: allow unknown fields
    Lenient,
    /// Auto mode: currently lenient, reserved for source-aware parsing later
    Auto,
    /// Backward-compatible name for lenient parsing in Rust callers
    #[serde(alias = "loose")]
    Loose,
}
