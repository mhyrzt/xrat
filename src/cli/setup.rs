use clap::{Args, ValueEnum};

#[derive(Debug, Args)]
#[command(
    about = "Run post-install setup: proxy cores, init, daemon, completions, man pages, xratui shortcut, and desktop integration."
)]
pub struct SetupArgs {
    /// Accept all recommended defaults without prompting.
    #[arg(short = 'y', long, conflicts_with = "check")]
    pub yes: bool,

    /// Do not install or start the background daemon.
    #[arg(long, conflicts_with = "check")]
    pub no_daemon: bool,

    /// Skip the desktop launcher and icon install (Linux/XDG only).
    #[arg(long, conflicts_with = "check")]
    pub no_desktop: bool,

    /// Skip installing shell completions.
    #[arg(long, conflicts_with = "check")]
    pub no_completions: bool,

    /// Skip installing man pages.
    #[arg(long, conflicts_with = "check")]
    pub no_manpages: bool,

    /// Enable boot-before-login start (Linux; implies installing the daemon).
    #[arg(long, conflicts_with = "check", conflicts_with = "no_daemon")]
    pub linger: bool,

    /// Diagnose only: report what is and is not set up, changing nothing.
    #[arg(long)]
    pub check: bool,

    /// Output format for --check.
    #[arg(long = "format", value_enum, default_value_t)]
    pub format: SetupFormat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum SetupFormat {
    #[default]
    Table,
    Json,
}
