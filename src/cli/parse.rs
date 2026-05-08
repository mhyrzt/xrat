use std::path::PathBuf;

use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Args)]
pub struct ParseArgs {
    #[arg(help = "Single config URI/text to parse.")]
    pub input: Option<String>,

    #[arg(long = "file", help = "Read config links from a local file.")]
    pub file: Option<PathBuf>,

    #[arg(long = "stdin", help = "Read config links from stdin.")]
    pub stdin: bool,

    #[arg(long = "json", help = "Print generated runtime JSON for one config.")]
    pub json: bool,

    #[arg(
        long = "engine",
        default_value_t,
        help = "Runtime engine selection mode."
    )]
    pub engine: ParseEngine,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ParseEngine {
    #[default]
    Auto,
    Xray,
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
