use std::io;

use clap::CommandFactory;
use clap_complete::generate;

use crate::app::context::AppContext;
use crate::cli::{Cli, CompletionsArgs};

pub fn run(_context: &AppContext, args: &CompletionsArgs) -> crate::app::Result<()> {
    let mut cmd = Cli::command();
    generate(args.shell, &mut cmd, "xrat", &mut io::stdout());
    Ok(())
}
