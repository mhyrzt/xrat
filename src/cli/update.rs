use clap::Args;

#[derive(Debug, Args, Default)]
#[command(about = "Refresh stored subscriptions.")]
pub struct UpdateArgs {
    #[arg(help = "Optional subscription ID or ref prefix list to refresh.")]
    pub subs_ref: Vec<String>,
}
