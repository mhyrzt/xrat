use clap::Args;

#[derive(Debug, Args)]
pub struct ConnectArgs {
    #[arg(help = "Stored config id to run as the active local proxy session.")]
    pub id: i64,
}
