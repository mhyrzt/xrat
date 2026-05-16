use clap::Args;

#[derive(Debug, Args, Default)]
#[command(about = "Stop the active managed proxy runtime.")]
pub struct DisconnectArgs {
    #[arg(long = "json", help = "Print the result as JSON.")]
    pub json: bool,
}
