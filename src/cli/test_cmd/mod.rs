use std::path::PathBuf;

use clap::Args;

use crate::db::ConfigListFilter;

mod enums;

pub use enums::{TestFormat, TestSortBy};

#[derive(Debug, Clone, Args)]
#[command(
    about = "Test connectivity and latency for stored configs.",
    long_about = "Test one config by ID, or bulk-test a filtered set of configs.\n\
        Without an ID, tests all configs matching the filter flags.\n\
        Results are printed to stdout (or to --output file) in the selected format."
)]
pub struct TestArgs {
    #[arg(help = "Config ID to test. Omit to bulk-test matching configs.")]
    pub id: Option<i64>,

    // -- Filter flags --
    #[arg(long = "enabled-only", help = "Filter: only enabled configs.")]
    pub enabled_only: bool,
    #[arg(long = "active-only", help = "Filter: only the active config.")]
    pub active_only: bool,
    #[arg(long = "selected-only", help = "Filter: only the selected config.")]
    pub selected_only: bool,
    #[arg(
        long = "subscription",
        help = "Filter: only configs from the given subscription ID."
    )]
    pub subscription: Option<i64>,

    // -- Skip test stages --
    #[arg(long = "skip-icmp", help = "Skip the ICMP ping stage.")]
    pub skip_icmp: bool,
    #[arg(long = "skip-tcp", help = "Skip the TCP connectivity stage.")]
    pub skip_tcp: bool,
    #[arg(
        long = "skip-real-delay",
        help = "Skip the real-delay (HTTP round-trip) stage."
    )]
    pub skip_real_delay: bool,
    #[arg(long = "skip-download", help = "Skip the download speed stage.")]
    pub skip_download: bool,
    #[arg(
        long = "skip-upload",
        help = "Skip the upload speed stage (disabled by default)."
    )]
    pub skip_upload: bool,

    // -- URL overrides --
    #[arg(
        long = "test-url",
        help = "Override the URL used for real-delay (HTTP round-trip) checks."
    )]
    pub test_url: Option<String>,
    #[arg(
        long = "download-url",
        help = "Override the URL used for download speed checks."
    )]
    pub download_url: Option<String>,
    #[arg(
        long = "upload-url",
        help = "Enable upload speed stage and set the HTTP POST target URL."
    )]
    pub upload_url: Option<String>,

    // -- Timeout overrides --
    #[arg(long = "icmp-timeout", help = "Override ICMP timeout in milliseconds.")]
    pub icmp_timeout_ms: Option<u64>,
    #[arg(
        long = "tcp-timeout",
        help = "Override TCP connect timeout in milliseconds."
    )]
    pub tcp_timeout_ms: Option<u64>,
    #[arg(
        long = "real-delay-timeout",
        help = "Override real-delay HTTP request timeout in milliseconds."
    )]
    pub real_delay_timeout_ms: Option<u64>,
    #[arg(
        long = "download-timeout",
        help = "Override download speed request timeout in milliseconds."
    )]
    pub download_timeout_ms: Option<u64>,
    #[arg(
        long = "upload-timeout",
        help = "Override upload speed request timeout in milliseconds."
    )]
    pub upload_timeout_ms: Option<u64>,

    // -- Concurrency and output --
    #[arg(long = "concurrency", help = "Bulk-test concurrency. 0 = auto-detect.")]
    pub concurrency: Option<i32>,
    #[arg(
        long = "format",
        default_value_t,
        help = "Output format for bulk results [default: tsv]."
    )]
    pub format: TestFormat,
    #[arg(
        long = "output",
        help = "Write bulk results to a file instead of stdout."
    )]
    pub output: Option<PathBuf>,
    #[arg(
        long = "sort-by",
        default_value_t,
        help = "Sort order for bulk results [default: status]."
    )]
    pub sort_by: TestSortBy,
    #[arg(long = "no-progress", help = "Hide the animated progress bar.")]
    pub no_progress: bool,

    // -- Ping loop --
    #[arg(
        long = "ping",
        help = "Continuously ping one config until Ctrl+C, printing a live summary."
    )]
    pub ping: bool,
    #[arg(
        long = "ping-interval",
        help = "Interval between ping-loop iterations in milliseconds.",
        default_value_t = 1000
    )]
    pub ping_interval_ms: u64,

    // -- Historical summary --
    #[arg(
        long = "latest-run-summary",
        help = "Print a summary of the latest persisted test run and exit."
    )]
    pub latest_run_summary: bool,
    #[arg(
        long = "country",
        help = "Filter latest-run summary by endpoint country ISO code (e.g. US, DE)."
    )]
    pub country: Option<String>,
    #[arg(
        long = "asn",
        help = "Filter latest-run summary by ASN (case-insensitive substring match)."
    )]
    pub asn: Option<String>,
}

impl TestArgs {
    pub fn config_filter(&self) -> ConfigListFilter {
        ConfigListFilter {
            only_enabled: self.enabled_only,
            only_selected: self.selected_only,
            only_active: self.active_only,
            only_deleted: false,
            include_deleted: false,
            subscription_id: self.subscription,
            protocol: None,
        }
    }
}
