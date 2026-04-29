use clap::Args;

#[derive(Debug, Args, Default)]
pub struct DisconnectArgs {
    #[arg(long = "json", help = "Print disconnect result as JSON.")]
    pub json: bool,
}
