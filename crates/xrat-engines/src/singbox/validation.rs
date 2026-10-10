use std::{path::Path, sync::Arc};
use xrat_support::process::{Command, ProcessSpawner};

/// Build the native syntax check used by managed runtime preflight.
pub fn config_check_command(
    binary: &Path,
    config: &Path,
    spawner: Arc<dyn ProcessSpawner>,
) -> Command {
    let mut command = Command::with_spawner(binary, spawner);
    command.args(["check", "-c"]).arg(config);
    command
}
