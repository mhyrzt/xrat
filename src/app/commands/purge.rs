use crate::app::commands::output;
use crate::app::context::AppContext;
use crate::cli::PurgeArgs;

pub async fn run(context: &AppContext, args: &PurgeArgs) -> crate::app::Result<()> {
    let pending = context.db.count_deleted_configs().await?;
    if pending == 0 {
        println!(
            "{}",
            output::notice("No soft-deleted configs to purge.", output::color_enabled())
        );
        return Ok(());
    }

    if !args.yes
        && !output::confirm(format!(
            "Permanently delete {pending} soft-deleted config(s)?"
        ))?
    {
        println!("{}", output::notice("Aborted.", output::color_enabled()));
        return Ok(());
    }

    let purged = context.db.purge_deleted_configs().await?;
    println!(
        "{}",
        output::success(
            format!("Purged {purged} config(s)."),
            output::color_enabled()
        )
    );
    Ok(())
}
