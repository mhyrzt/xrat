use std::path::PathBuf;

use clap::{Args, ValueEnum};

use crate::db::ConfigListFilter;

#[derive(Debug, Clone, Args)]
pub struct TestArgs {
    #[arg(help = "Config ID to test.")]
    pub id: Option<i64>,

    #[arg(long = "enabled-only", help = "Bulk test only enabled configs.")]
    pub enabled_only: bool,

    #[arg(long = "active-only", help = "Bulk test only the active config.")]
    pub active_only: bool,

    #[arg(long = "selected-only", help = "Bulk test only the selected config.")]
    pub selected_only: bool,

    #[arg(
        long = "subscription",
        help = "Bulk test only configs from one subscription id."
    )]
    pub subscription: Option<i64>,

    #[arg(long = "skip-icmp", help = "Skip ICMP ping test.")]
    pub skip_icmp: bool,

    #[arg(long = "skip-tcp", help = "Skip TCP connectivity test.")]
    pub skip_tcp: bool,

    #[arg(long = "skip-real-delay", help = "Skip real-delay test.")]
    pub skip_real_delay: bool,

    #[arg(long = "skip-download", help = "Skip download speed test.")]
    pub skip_download: bool,

    #[arg(
        long = "test-url",
        help = "Override the URL used for real-delay checks."
    )]
    pub test_url: Option<String>,

    #[arg(
        long = "download-url",
        help = "Override the URL used for download speed checks."
    )]
    pub download_url: Option<String>,

    #[arg(
        long = "icmp-timeout",
        help = "Override the ICMP timeout in milliseconds."
    )]
    pub icmp_timeout_ms: Option<u64>,

    #[arg(
        long = "tcp-timeout",
        help = "Override the TCP timeout in milliseconds."
    )]
    pub tcp_timeout_ms: Option<u64>,

    #[arg(
        long = "real-delay-timeout",
        help = "Override the real-delay request timeout in milliseconds."
    )]
    pub real_delay_timeout_ms: Option<u64>,

    #[arg(
        long = "download-timeout",
        help = "Override the download speed request timeout in milliseconds."
    )]
    pub download_timeout_ms: Option<u64>,

    #[arg(long = "concurrency", help = "Bulk test concurrency. 0 means auto.")]
    pub concurrency: Option<i32>,

    #[arg(long = "format", default_value_t, help = "Bulk result output format.")]
    pub format: TestFormat,

    #[arg(
        long = "output",
        help = "Write bulk results to a file instead of stdout."
    )]
    pub output: Option<PathBuf>,

    #[arg(long = "sort-by", default_value_t, help = "Sort bulk results.")]
    pub sort_by: TestSortBy,

    #[arg(long = "no-progress", help = "Disable bulk progress output.")]
    pub no_progress: bool,

    #[arg(
        long = "latest-run-summary",
        help = "Print summary for latest persisted test run and exit."
    )]
    pub latest_run_summary: bool,

    #[arg(
        long = "country",
        help = "Filter latest-run summary by endpoint country (ISO code)."
    )]
    pub country: Option<String>,

    #[arg(
        long = "asn",
        help = "Filter latest-run summary by endpoint ASN text (case-insensitive substring)."
    )]
    pub asn: Option<String>,
}

impl TestArgs {
    pub fn config_filter(&self) -> ConfigListFilter {
        ConfigListFilter {
            only_enabled: self.enabled_only,
            only_selected: self.selected_only,
            only_active: self.active_only,
            subscription_id: self.subscription,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum TestFormat {
    #[default]
    Tsv,
    Csv,
    Json,
}

impl std::fmt::Display for TestFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tsv => formatter.write_str("tsv"),
            Self::Csv => formatter.write_str("csv"),
            Self::Json => formatter.write_str("json"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum TestSortBy {
    #[default]
    Status,
    Icmp,
    RealDelay,
    DownloadSpeed,
    Protocol,
    Address,
}

impl std::fmt::Display for TestSortBy {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Status => formatter.write_str("status"),
            Self::Icmp => formatter.write_str("icmp"),
            Self::RealDelay => formatter.write_str("real-delay"),
            Self::DownloadSpeed => formatter.write_str("download-speed"),
            Self::Protocol => formatter.write_str("protocol"),
            Self::Address => formatter.write_str("address"),
        }
    }
}
