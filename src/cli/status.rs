use clap::Args;

#[derive(Debug, Args, Default)]
pub struct StatusArgs {
    #[arg(long = "json", help = "Print runtime status as JSON.")]
    pub json: bool,
}
