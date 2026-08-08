mod check;
mod errors;
mod status;

pub use check::{RealDelayResult, real_delay_check};
pub use status::AcceptedHttpStatuses;

#[cfg(test)]
mod tests;
