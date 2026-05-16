use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Import a subscription URL, file, or raw text into SQLite.")]
pub struct ImportArgs {
    #[arg(help = "Subscription source: a URL, local file path, or raw subscription text.")]
    pub input: String,
}
