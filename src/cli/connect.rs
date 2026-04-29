use clap::Args;

#[derive(Debug, Args)]
pub struct ConnectArgs {
    #[arg(help = "Stored config id to run as the active local proxy session.")]
    pub id: i64,

    #[arg(long = "json", help = "Print connect result as JSON.")]
    pub json: bool,
}
