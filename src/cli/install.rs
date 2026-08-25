use clap::{Args, ValueEnum};
use semver::Version;

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Proxy core to install from its upstream GitHub repository.
    #[arg(value_enum)]
    pub core: InstallCore,

    /// Release version to install. Defaults to the latest stable release.
    #[arg(
        long,
        value_name = "VERSION",
        value_parser = parse_version,
        conflicts_with = "prerelease"
    )]
    pub version: Option<Version>,

    /// Install the newest published prerelease instead of the latest stable release.
    #[arg(long, conflicts_with = "version")]
    pub prerelease: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum InstallCore {
    Xray,
    #[value(name = "v2ray")]
    V2Ray,
    #[value(name = "sing-box", alias = "singbox")]
    SingBox,
}

fn parse_version(value: &str) -> Result<Version, semver::Error> {
    Version::parse(value.trim_start_matches('v'))
}
