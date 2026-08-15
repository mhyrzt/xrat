mod check;
mod errors;
mod status;

pub(crate) use check::request::make_proxied_request_via;
pub use check::{RealDelayResult, real_delay_check};
pub use status::AcceptedHttpStatuses;

#[cfg(test)]
mod tests;
