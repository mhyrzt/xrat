use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::cli::{DbAction, DbArgs};

pub async fn run(context: &AppContext, args: &DbArgs) -> crate::app::Result<()> {
    match &args.action {
        DbAction::Migrate(_) => migrate(context),
    }
}

/// Report migration status. Migrations are applied while building the
/// [`AppContext`] before this handler runs, so reaching this point means they
/// completed successfully; a failure would have surfaced earlier (with the
/// actionable message from `src/db/schema.rs`) and aborted the command.
fn migrate(_context: &AppContext) -> crate::app::Result<()> {
    println!(
        "{}",
        output::success(
            "Database migrations are up to date.",
            output::color_enabled()
        )
    );
    Ok(())
}
