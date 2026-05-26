use clap::Args;

#[derive(Debug, Args)]
pub struct ServeArgs {
    #[arg(long, help = "Override HTTP API bind host.")]
    pub host: Option<String>,
    #[arg(long, help = "Override HTTP API bind port.")]
    pub port: Option<u16>,
}
