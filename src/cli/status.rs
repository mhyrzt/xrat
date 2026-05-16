use clap::Args;

#[derive(Debug, Args, Default)]
#[command(about = "Show the managed proxy runtime status.")]
pub struct StatusArgs {
    #[arg(long = "json", help = "Print the status as JSON.")]
    pub json: bool,
}
