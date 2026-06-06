use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Start a managed proxy runtime for a stored config.")]
pub struct ConnectArgs {
    #[arg(help = "Config ID or ref prefix to start as the active local proxy session.")]
    pub id: String,

    #[arg(long = "json", help = "Print the result as JSON.")]
    pub json: bool,
}
