use clap::ValueEnum;

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
