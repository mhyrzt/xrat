mod process;
mod signals;

pub use process::{
    ManagedXrayPaths, ManagedXrayProcess, XrayRuntimeError, process_is_running, spawn_detached,
    spawn_detached_json_with_ports, spawn_detached_with_ports,
};
pub use signals::{
    TerminationOutcome, XraySignalError, terminate_process, terminate_process_gracefully,
    terminate_process_gracefully_with_signals,
};

#[cfg(test)]
mod tests;
