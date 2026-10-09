use clap::{Args, Subcommand, ValueEnum};

use crate::cli::ListFormat;

#[derive(Debug, Args)]
#[command(about = "Enable, disable, and inspect managed TUN capture and split tunneling.")]
pub struct TunArgs {
    #[command(subcommand)]
    pub action: TunAction,
}

#[derive(Debug, Subcommand)]
pub enum TunAction {
    #[command(
        about = "Enable TUN on the current connection, or on the next connect when disconnected."
    )]
    Enable(TunModeArgs),
    #[command(about = "Disable TUN and keep the current config connected in proxy mode.")]
    Disable(TunModeArgs),
    #[command(about = "Set the TUN split tunneling mode (all, blacklist, or whitelist).")]
    Mode(TunSetSplitModeArgs),
    #[command(
        about = "Manage per-application TUN split tunneling lists (blacklist and whitelist)."
    )]
    Split(TunSplitArgs),
    #[command(about = "Report configured mode, active capture, engine support, and privileges.")]
    Status(TunStatusArgs),
    #[command(
        about = "Grant CAP_NET_ADMIN/CAP_NET_RAW/CAP_DAC_READ_SEARCH/CAP_SYS_PTRACE to the files TUN needs via setcap."
    )]
    Setup(TunSetupArgs),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TunSplitModeArg {
    All,
    Blacklist,
    Whitelist,
}

impl From<TunSplitModeArg> for crate::app::config::TunSplitMode {
    fn from(value: TunSplitModeArg) -> Self {
        match value {
            TunSplitModeArg::All => Self::All,
            TunSplitModeArg::Blacklist => Self::Blacklist,
            TunSplitModeArg::Whitelist => Self::Whitelist,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TunSplitListTarget {
    Blacklist,
    Whitelist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TunSplitClearTarget {
    Blacklist,
    Whitelist,
    Both,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum TunSplitListViewTarget {
    #[default]
    All,
    Blacklist,
    Whitelist,
}

#[derive(Debug, Args, Default)]
pub struct TunStatusArgs {
    #[arg(long = "json", help = "Print the result as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args, Default)]
pub struct TunSetupArgs {
    #[arg(long, help = "Print the setcap command without running it.")]
    pub dry_run: bool,
}

#[derive(Debug, Args, Default)]
pub struct TunModeArgs {
    #[arg(
        long,
        value_enum,
        help = "Optional split tunneling mode to set while enabling TUN."
    )]
    pub mode: Option<TunSplitModeArg>,
    #[arg(long, help = "Print the applied TUN state as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct TunSetSplitModeArgs {
    #[arg(
        value_enum,
        help = "Split tunneling mode: all, blacklist, or whitelist."
    )]
    pub mode: TunSplitModeArg,
    #[arg(long, help = "Print the applied TUN state as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct TunSplitArgs {
    #[command(subcommand)]
    pub action: TunSplitAction,
}

#[derive(Debug, Subcommand)]
pub enum TunSplitAction {
    #[command(about = "Show current split tunneling mode, lists, and resolved engine rules.")]
    Status(TunStatusArgs),
    #[command(about = "Set split tunneling mode (all, blacklist, or whitelist).")]
    Mode(TunSetSplitModeArgs),
    #[command(about = "List configured applications in blacklist and/or whitelist.")]
    List(TunSplitListArgs),
    #[command(about = "Add one or more applications to blacklist or whitelist.")]
    Add(TunSplitModifyArgs),
    #[command(about = "Remove one or more applications from blacklist or whitelist.")]
    Remove(TunSplitModifyArgs),
    #[command(about = "Clear entries from blacklist, whitelist, or both.")]
    Clear(TunSplitClearArgs),
    #[command(about = "Discover running processes and installed desktop applications.")]
    Apps(TunSplitAppsArgs),
}

#[derive(Debug, Args, Default)]
pub struct TunSplitListArgs {
    #[arg(
        value_enum,
        default_value_t = TunSplitListViewTarget::All,
        help = "Which list to display (all, blacklist, or whitelist)."
    )]
    pub list: TunSplitListViewTarget,
    #[arg(
        long,
        value_enum,
        default_value_t = ListFormat::Table,
        help = "Output format (table, tsv, or json)."
    )]
    pub format: ListFormat,
    #[arg(long, help = "Print the result as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct TunSplitModifyArgs {
    #[arg(value_enum, help = "Target list: blacklist or whitelist.")]
    pub list: TunSplitListTarget,
    #[arg(
        required = true,
        num_args = 1..,
        help = "Application specifiers (binary name, /abs/path, /abs/dir/, or .desktop ID)."
    )]
    pub apps: Vec<String>,
    #[arg(long, help = "Print the result as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct TunSplitClearArgs {
    #[arg(
        value_enum,
        help = "Target list to clear: blacklist, whitelist, or both."
    )]
    pub list: TunSplitClearTarget,
    #[arg(long, help = "Print the result as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args, Default)]
pub struct TunSplitAppsArgs {
    #[arg(long, help = "Include only running user processes.")]
    pub running: bool,
    #[arg(long, help = "Include only installed .desktop applications.")]
    pub desktop: bool,
    #[arg(
        long,
        help = "Filter discovered applications by case-insensitive substring."
    )]
    pub search: Option<String>,
    #[arg(
        long,
        value_enum,
        default_value_t = ListFormat::Table,
        help = "Output format (table, tsv, or json)."
    )]
    pub format: ListFormat,
    #[arg(long, help = "Print the result as JSON.")]
    pub json: bool,
}
