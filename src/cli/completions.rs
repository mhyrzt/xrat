use clap::Args;
use clap_complete::Shell;

#[derive(Debug, Args)]
#[command(about = "Generate shell completion scripts for xrat.")]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: Shell,
}
