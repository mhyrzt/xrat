use std::path::PathBuf;

use clap::{Args, Subcommand};

#[derive(Debug, Args)]
#[command(about = "Inspect and manage GeoLite2 MMDB assets.")]
pub struct GeoIpArgs {
    #[command(subcommand)]
    pub action: GeoIpAction,
}

#[derive(Debug, Subcommand)]
pub enum GeoIpAction {
    #[command(about = "Print the resolved MMDB directory.")]
    Path(GeoIpPathArgs),
    #[command(about = "Show MMDB presence and size for each supported edition.")]
    Status(GeoIpStatusArgs),
}

#[derive(Debug, Args, Default)]
pub struct GeoIpPathArgs {
    #[arg(
        long = "output",
        help = "Override the MMDB target directory for this command."
    )]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args, Default)]
pub struct GeoIpStatusArgs {
    #[arg(
        long = "output",
        help = "Override the MMDB target directory for this command."
    )]
    pub output: Option<PathBuf>,
    #[arg(
        long = "strict",
        help = "Exit non-zero when any supported edition is missing."
    )]
    pub strict: bool,
}
