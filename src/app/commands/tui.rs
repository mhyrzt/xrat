use crate::app::context::AppContext;
use crate::cli::TuiArgs;

pub async fn run(context: &AppContext, _args: &TuiArgs) -> crate::app::Result<()> {
    crate::tui::run(context).await
}
