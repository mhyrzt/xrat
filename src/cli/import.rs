use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Import a subscription URL, file, or raw text into SQLite.")]
pub struct ImportArgs {
    #[arg(help = "Subscription source: a URL, local file path, or raw subscription text.")]
    pub input: String,

    #[arg(
        short = 'n',
        long,
        value_name = "NAME",
        value_parser = parse_name,
        help = "Name for the imported subscription source."
    )]
    pub name: Option<String>,
}

fn parse_name(value: &str) -> Result<String, String> {
    let name = value.trim();
    if name.is_empty() {
        return Err("name must not be empty".to_string());
    }
    Ok(name.to_string())
}
