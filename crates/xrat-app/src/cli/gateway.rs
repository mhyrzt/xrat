use clap::{Args, Subcommand};

#[derive(Debug, Args)]
pub struct GatewayArgs {
    #[command(subcommand)]
    pub action: GatewayAction,
}

#[derive(Debug, Subcommand)]
pub enum GatewayAction {
    /// Inspect gateway prerequisites without changing the system.
    Doctor(GatewayReadArgs),
    /// Install gateway protection and configuration.
    Install(GatewayInstallArgs),
    /// Inspect gateway installation and observed protection.
    Status(GatewayReadArgs),
    /// Safely remove owned gateway resources.
    Remove(GatewayRemoveArgs),
}

#[derive(Debug, Args, Default)]
pub struct GatewayReadArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Default)]
pub struct GatewayInstallArgs {
    #[arg(long)]
    pub interface: Option<String>,
    #[arg(long)]
    pub lan_subnet: Option<String>,
    #[arg(long = "local-subnet")]
    pub local_subnets: Vec<String>,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args, Default)]
pub struct GatewayRemoveArgs {
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
}
