use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum TestFormat {
    /// Tab-separated values (default, easy to read in terminal).
    #[default]
    Tsv,
    /// Comma-separated values (spreadsheet compatible).
    Csv,
    /// JSON output (machine-parseable).
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
    /// Sort by test result status (alive first, then by failure reason).
    #[default]
    Status,
    /// Sort by ICMP latency (lowest first).
    Icmp,
    /// Sort by real-delay latency (lowest first).
    RealDelay,
    /// Sort by download throughput (highest first).
    DownloadSpeed,
    /// Sort by protocol type alphabetically.
    Protocol,
    /// Sort by server address alphabetically.
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
