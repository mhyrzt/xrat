mod check;
mod errors;

pub use check::{RealDelayResult, real_delay_check};

#[cfg(test)]
mod tests;
