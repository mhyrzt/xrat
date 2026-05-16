mod process;
mod signals;

pub use process::{ManagedXrayPaths, ManagedXrayProcess, process_is_running, spawn_detached};
pub use signals::{TerminationOutcome, terminate_process, terminate_process_gracefully};

#[cfg(test)]
mod tests;
