use clap::Args;

#[derive(Debug, Clone, Args)]
pub struct TestArgs {
    /// Config ID to test
    pub id: i64,

    /// Skip ICMP ping test
    #[arg(long)]
    pub skip_icmp: bool,

    /// Skip TCP connectivity test
    #[arg(long)]
    pub skip_tcp: bool,

    /// Skip real-delay test
    #[arg(long)]
    pub skip_real_delay: bool,

    /// Custom test URL for real-delay check
    #[arg(long)]
    pub test_url: Option<String>,
}
