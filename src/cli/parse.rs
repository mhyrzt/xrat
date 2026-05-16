use std::path::PathBuf;

use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Args)]
#[command(
    about = "Parse and validate config links without importing to the database.",
    long_about = "Parse one or more config links (vless://, vmess://, ss://, etc.)\n\
        and print the decoded fields. Provide input via positional arg, --file, or --stdin."
)]
pub struct ParseArgs {
    #[arg(help = "Single config URI to parse, e.g. vless://... or vmess://...")]
    pub input: Option<String>,

    #[arg(
        long = "file",
        help = "Read config links (one per line) from a local file."
    )]
    pub file: Option<PathBuf>,

    #[arg(long = "stdin", help = "Read config links (one per line) from stdin.")]
    pub stdin: bool,

    #[arg(
        long = "json",
        help = "Print the generated runtime JSON config for the parsed node."
    )]
    pub json: bool,

    #[arg(
        long = "engine",
        default_value_t,
        help = "Proxy engine used to generate runtime config [default: auto]."
    )]
    pub engine: ParseEngine,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ParseEngine {
    /// Auto-detect: uses sing-box for hysteria2, xray for everything else.
    #[default]
    Auto,
    /// Always use Xray / Xray-core to generate the config.
    Xray,
    /// Always use sing-box to generate the config.
    #[value(name = "sing-box")]
    SingBox,
}

impl std::fmt::Display for ParseEngine {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => formatter.write_str("auto"),
            Self::Xray => formatter.write_str("xray"),
            Self::SingBox => formatter.write_str("sing-box"),
        }
    }
}
