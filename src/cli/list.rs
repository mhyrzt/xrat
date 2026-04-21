use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Args)]
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
    #[arg(long = "all", help = "Include deleted configs in the result.")]
    pub all: bool,
    #[arg(long = "deleted", help = "Show only deleted configs.")]
    pub deleted: bool,
    #[arg(long = "enabled-only", help = "Show only enabled configs.")]
    pub enabled_only: bool,
    #[arg(long = "active-only", help = "Show only the active config.")]
    pub active_only: bool,
    #[arg(long = "selected-only", help = "Show only the selected config.")]
    pub selected_only: bool,
    #[arg(
        long = "subscription",
        help = "Show only configs from one subscription id."
    )]
    pub subscription: Option<i64>,
}

#[derive(Debug, Args, Default)]
pub struct ListSubscriptionsArgs {
    #[arg(long = "kind", help = "Filter subscriptions by source kind.")]
    pub kind: Option<SubscriptionKind>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum SubscriptionKind {
    Url,
    File,
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
