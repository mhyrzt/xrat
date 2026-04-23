pub mod core;
pub mod protocols;
pub mod shared;
pub mod transports;

pub use core::*;
pub use protocols::*;
pub use shared::*;
pub use transports::*;

/// Parsing mode for Xray configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseMode {
    /// Strict mode: reject unknown fields
    Strict,
    /// Loose mode: allow unknown fields
    Loose,
}
