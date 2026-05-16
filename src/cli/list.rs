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
    #[arg(long = "selected-only", help = "Show only the selected config.")]
    pub selected_only: bool,
    #[arg(
        long = "subscription",
        help = "Show only configs from the given subscription ID."
    )]
    pub subscription: Option<i64>,
}

#[derive(Debug, Args, Default)]
pub struct ListSubscriptionsArgs {
    #[arg(
        long = "kind",
        help = "Filter by source kind: url (remote subscription link), file (local file path), or raw-text (inline text)."
    )]
    pub kind: Option<SubscriptionKind>,
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
