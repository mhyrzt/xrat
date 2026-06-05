use crate::app::context::AppContext;
use crate::cli::VersionArgs;

pub fn run(_context: &AppContext, _args: &VersionArgs) -> crate::app::Result<()> {
    println!("xrat {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
