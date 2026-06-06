use std::fmt;

use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Args)]
#[command(about = "List persisted nodes or subscriptions.")]
pub struct ListArgs {
    #[command(subcommand)]
    pub target: ListTarget,
}

#[derive(Debug, Subcommand)]
pub enum ListTarget {
    #[command(alias = "nodes", about = "List stored nodes/configs.")]
    Configs(ListConfigsArgs),
    #[command(alias = "subs", about = "List stored subscription sources.")]
    Subscriptions(ListSubscriptionsArgs),
}

#[derive(Debug, Args)]
pub struct ListConfigsArgs {
    #[arg(long = "enabled-only", help = "Show only enabled configs.")]
    pub enabled_only: bool,
    #[arg(long = "active-only", help = "Show only the active config.")]
    pub active_only: bool,
    #[arg(long = "deleted", help = "Show only soft-deleted configs.")]
    pub deleted_only: bool,
    #[arg(long = "all", help = "Include soft-deleted configs in results.")]
    pub include_deleted: bool,
    #[arg(
        long = "subscription",
        help = "Show only configs from the given subscription ID or ref prefix."
    )]
    pub subscription: Option<String>,
    #[arg(
        long = "format",
        value_enum,
        default_value_t,
        help = "Output format: table, tsv, or json [default: table]."
    )]
    pub format: ListFormat,
}

#[derive(Debug, Args, Default)]
pub struct ListSubscriptionsArgs {
    #[arg(
        long = "kind",
        help = "Filter by source kind: url (remote subscription link), file (local file path), or raw-text (inline text)."
    )]
    pub kind: Option<SubscriptionKind>,
    #[arg(
        long = "format",
        value_enum,
        default_value_t,
        help = "Output format: table, tsv, or json [default: table]."
    )]
    pub format: ListFormat,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum ListFormat {
    /// Aligned table for terminals.
    #[default]
    Table,
    /// Tab-separated values for scripts.
    Tsv,
    /// Pretty JSON.
    Json,
}

impl fmt::Display for ListFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Table => formatter.write_str("table"),
            Self::Tsv => formatter.write_str("tsv"),
            Self::Json => formatter.write_str("json"),
        }
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SubscriptionKind {
    /// Remote subscription URL (https://...).
    Url,
    /// Local file path on disk.
    File,
    /// Inline raw subscription text.
    RawText,
}

impl SubscriptionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Url => "url",
            Self::File => "file",
            Self::RawText => "raw_text",
        }
    }
}
