use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::cli::Command;

#[derive(Debug, Parser)]
#[command(about = "Manage XRAT configs and persisted app state.")]
pub struct Cli {
    #[arg(
        short = 'v',
        long = "verbose",
        global = true,
        action = ArgAction::Count,
        help = "Increase diagnostic logging verbosity. Repeat for debug/trace output."
    )]
    pub verbose: u8,
    #[arg(
        short = 'q',
        long = "quiet",
        global = true,
        help = "Show only error diagnostics unless RUST_LOG is set."
    )]
    pub quiet: bool,
    #[arg(
        long = "database",
        global = true,
        help = "Override the SQLite database path. Defaults to [database.sqlite].path, XRAT_PATH/db.sqlite, or $HOME/.config/xrat/db.sqlite."
    )]
    pub database: Option<PathBuf>,
    #[arg(
        long = "config",
        global = true,
        help = "Override the config file path. Defaults to XRAT_PATH/config.toml or $HOME/.config/xrat/config.toml."
    )]
    pub config: Option<PathBuf>,
    #[arg(
        long = "xray",
        global = true,
        help = "Override the Xray binary path. Defaults to [paths].xray in config.toml or xray."
    )]
    pub xray: Option<PathBuf>,
    #[arg(
        long = "v2ray",
        global = true,
        help = "Override the V2Ray binary path. Defaults to [paths].v2ray in config.toml or v2ray."
    )]
    pub v2ray: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn default_log_filter(&self) -> &'static str {
        if self.quiet {
            return "error";
        }

        match self.verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    }
}
