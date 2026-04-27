use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct TestArgs {
    #[arg(help = "Config ID to test.")]
    pub id: i64,

    #[arg(long = "skip-icmp", help = "Skip ICMP ping test.")]
    pub skip_icmp: bool,

    #[arg(long = "skip-tcp", help = "Skip TCP connectivity test.")]
    pub skip_tcp: bool,

    #[arg(long = "skip-real-delay", help = "Skip real-delay test.")]
    pub skip_real_delay: bool,

    #[arg(
        long = "test-url",
        help = "Override the URL used for real-delay checks."
    )]
    pub test_url: Option<String>,

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
}
