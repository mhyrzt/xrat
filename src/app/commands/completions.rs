use std::io::{self, Write};

use clap::CommandFactory;
use clap_complete::{Shell, generate};

use crate::app::context::AppContext;
use crate::cli::{Cli, CompletionsArgs};

pub fn run(_context: &AppContext, args: &CompletionsArgs) -> crate::app::Result<()> {
    generate_to(args.shell, &mut io::stdout());
    Ok(())
}

/// Write the completion script for `shell` to an arbitrary sink. Used by `run`
/// (stdout) and by `setup` (installs to per-shell completion directories).
pub fn generate_to(shell: Shell, writer: &mut impl Write) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "xrat", writer);
}
