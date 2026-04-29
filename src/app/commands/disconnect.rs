use crate::app::runtime::AppContext;
use crate::cli::DisconnectArgs;

pub async fn run(context: &AppContext, _args: &DisconnectArgs) -> crate::app::Result<()> {
    if super::runtime_lifecycle::stop_active_session(context).await? {
        println!("Disconnected active runtime session");
    } else {
        println!("No active runtime session");
    }
    Ok(())
}
