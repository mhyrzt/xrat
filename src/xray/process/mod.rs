mod errors;
mod spawn;

pub use errors::XrayProcessError;
pub use spawn::XrayProcess;

#[cfg(test)]
mod tests;
