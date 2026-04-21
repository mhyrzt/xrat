use clap::Args;

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[arg(help = "Subscription input: URL, file path, or raw subscription text.")]
    pub input: String,
}
