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
    #[command(about = "Download one or more GeoLite2 MMDB editions.")]
    Download(GeoIpDownloadArgs),
    #[command(about = "Refresh all supported GeoLite2 MMDB editions.")]
    Update(GeoIpUpdateArgs),
    #[command(about = "Print the resolved MMDB directory.")]
    Path(GeoIpPathArgs),
    #[command(about = "Show MMDB presence and size for each supported edition.")]
    Status(GeoIpStatusArgs),
}

#[derive(Debug, Args, Default)]
pub struct GeoIpDownloadArgs {
    #[arg(
        long = "edition",
        help = "Edition to download. Repeatable: GeoLite2-Country|GeoLite2-City|GeoLite2-ASN or country|city|asn."
    )]
    pub editions: Vec<String>,
    #[arg(long = "all", help = "Download all supported editions.")]
    pub all: bool,
    #[arg(
        long = "output",
        help = "Override the MMDB target directory for this command."
    )]
    pub output: Option<PathBuf>,
    #[arg(
        long = "force",
        help = "Re-download even when the destination file already exists."
    )]
    pub force: bool,
    #[arg(
        long = "url",
        help = "Override the download URL template. Use {edition} as a placeholder."
    )]
    pub url: Option<String>,
    #[arg(
        long = "timeout",
        help = "Override the HTTP request timeout in seconds."
    )]
    pub timeout_secs: Option<u64>,
    #[arg(long = "quiet", help = "Suppress progress bar output.")]
    pub quiet: bool,
}

#[derive(Debug, Args, Default)]
pub struct GeoIpUpdateArgs {
    #[arg(
        long = "output",
        help = "Override the MMDB target directory for this command."
    )]
    pub output: Option<PathBuf>,
    #[arg(
        long = "url",
        help = "Override the download URL template. Use {edition} as a placeholder."
    )]
    pub url: Option<String>,
    #[arg(
        long = "timeout",
        help = "Override the HTTP request timeout in seconds."
    )]
    pub timeout_secs: Option<u64>,
    #[arg(long = "quiet", help = "Suppress progress bar output.")]
    pub quiet: bool,
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
