use clap::{Args, ValueEnum};

use crate::cli::ListFormat;

#[derive(Debug, Args)]
#[command(about = "Show app events plus xray-core / sing-box engine logs.")]
pub struct LogsArgs {
    #[arg(
        short = 'f',
        long = "follow",
        help = "Stream new log entries as they arrive instead of exiting."
    )]
    pub follow: bool,
    #[arg(
        short = 'n',
        long = "lines",
        default_value_t = 200,
        help = "Number of recent entries to show before following [default: 200]."
    )]
    pub lines: usize,
    #[arg(
        long = "source",
        value_enum,
        default_value_t,
        help = "Which log feeds to include: all, app, daemon, xray, or singbox [default: all]."
    )]
    pub source: LogSource,
    #[arg(
        long = "level",
        value_enum,
        help = "Only show events at or above this level (applies to app/event entries)."
    )]
    pub level: Option<LogLevel>,
    #[arg(
        long = "format",
        value_enum,
        default_value_t,
        help = "Output format for the event stream: table, tsv, or json [default: table]."
    )]
    pub format: ListFormat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum LogSource {
    /// App events + engine logs + daemon log.
    #[default]
    All,
    /// Structured app/runtime events only.
    App,
    /// Daemon process log file only.
    Daemon,
    /// xray-core engine logs for the active/last session.
    Xray,
    /// sing-box engine logs for the active/last session.
    Singbox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}
