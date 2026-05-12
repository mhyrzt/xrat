#[cfg(unix)]
mod unix_impl;
#[cfg(not(unix))]
mod unsupported_impl;

#[cfg(unix)]
pub use unix_impl::*;
#[cfg(not(unix))]
pub use unsupported_impl::*;
