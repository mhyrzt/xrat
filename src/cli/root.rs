use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::cli::Command;

#[derive(Debug, Parser)]
#[command(
    name = "xrat",
    version,
    about = "XRAT -- proxy config manager and connection tester.",
    long_about = "XRAT imports, parses, tests, and manages proxy configs\n\
        (vless, vmess, ss, trojan, hysteria2) with a SQLite backend\n\
        and optional daemon supervisor for auto-rotation."
)]
pub struct Cli {
    #[arg(
        short = 'v',
        long = "verbose",
        global = true,
        action = ArgAction::Count,
        help = "Increase log verbosity. Repeat for more: -v = info, -vv = debug, -vvv = trace."
    )]
    pub verbose: u8,
    #[arg(
        short = 'q',
        long = "quiet",
        global = true,
        help = "Suppress all output except errors. Ignored if RUST_LOG is set."
    )]
    pub quiet: bool,
    #[arg(
        long = "database",
        global = true,
        help = "SQLite database path. Falls back to config.toml [database.sqlite].path, XRAT_PATH/db.sqlite, or ~/.config/xrat/db.sqlite."
    )]
    pub database: Option<PathBuf>,
    #[arg(
        long = "config",
        global = true,
        help = "Config file path. Falls back to XRAT_PATH/config.toml or ~/.config/xrat/config.toml."
    )]
    pub config: Option<PathBuf>,
    #[arg(
        long = "xray",
        global = true,
        help = "Xray binary path. Falls back to config.toml [paths].xray or 'xray' in PATH."
    )]
    pub xray: Option<PathBuf>,
    #[arg(
        long = "v2ray",
        global = true,
        help = "V2Ray binary path. Falls back to config.toml [paths].v2ray or 'v2ray' in PATH."
    )]
    pub v2ray: Option<PathBuf>,
    #[arg(
        long = "sing-box",
        global = true,
        help = "sing-box binary path. Falls back to config.toml [paths].sing_box or 'sing-box' in PATH."
    )]
    pub sing_box: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn default_log_filter(&self) -> &'static str {
        if self.quiet {
            return "error";
        }

        match self.verbose {
            0 => "error",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    }
}
