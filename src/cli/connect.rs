use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Start a managed proxy runtime for a stored config.")]
pub struct ConnectArgs {
    #[arg(help = "Config ID to start as the active local proxy session.")]
    pub id: i64,

    #[arg(long = "json", help = "Print the result as JSON.")]
    pub json: bool,
}
