use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct ScanArgs {
    #[arg(
        long = "ips",
        value_delimiter = ',',
        help = "Comma-separated IPs to scan, e.g. 1.1.1.1,8.8.8.8"
    )]
    pub ips: Vec<String>,

    #[arg(long = "file", help = "Read newline-separated IPs from file.")]
    pub file: Option<std::path::PathBuf>,

    #[arg(long = "port", default_value_t = 443, help = "Target TCP port.")]
    pub port: u16,

    #[arg(
        long = "timeout",
        default_value_t = 4000,
        help = "TCP timeout in milliseconds."
    )]
    pub timeout_ms: u64,

    #[arg(
        long = "history",
        help = "Print latest persisted scanner rows and exit."
    )]
    pub history: Option<i64>,
}
