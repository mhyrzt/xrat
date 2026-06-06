use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Args)]
#[command(about = "Local proxy endpoints and host/session integration helpers.")]
pub struct ProxyArgs {
    #[command(subcommand)]
    pub action: ProxyAction,
}

#[derive(Debug, Subcommand)]
pub enum ProxyAction {
    #[command(about = "Show active local proxy endpoints (HTTP, SOCKS5, Shadowsocks, PAC).")]
    Endpoints(ProxyEndpointsArgs),
    #[command(about = "Print or locate the Proxy Auto-Config (PAC) file.")]
    Pac(ProxyPacArgs),
    #[command(about = "Print shell commands to proxy the current terminal session.")]
    Shell(ProxyShellArgs),
    #[command(about = "Manage Linux desktop environment proxy settings.")]
    Desktop(ProxyDesktopArgs),
}

#[derive(Debug, Args, Default)]
pub struct ProxyEndpointsArgs {
    #[arg(long = "json", help = "Print endpoints as JSON.")]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ProxyPacArgs {
    #[command(subcommand)]
    pub action: ProxyPacAction,
}

#[derive(Debug, Subcommand)]
pub enum ProxyPacAction {
    #[command(about = "Print the PAC URL served by the API server.")]
    Url(ProxyPacUrlArgs),
    #[command(about = "Print the generated PAC file for the active runtime.")]
    Print(ProxyPacPrintArgs),
}

#[derive(Debug, Args, Default)]
pub struct ProxyPacUrlArgs {}

#[derive(Debug, Args, Default)]
pub struct ProxyPacPrintArgs {}

#[derive(Debug, Args)]
pub struct ProxyShellArgs {
    #[command(subcommand)]
    pub action: ProxyShellAction,
}

#[derive(Debug, Subcommand)]
pub enum ProxyShellAction {
    #[command(about = "Print commands that point the current shell at xrat endpoints.")]
    Enable(ProxyShellEnableArgs),
    #[command(about = "Print commands that unset the proxy environment variables.")]
    Disable(ProxyShellDisableArgs),
    #[command(about = "Report whether the current shell points at active xrat endpoints.")]
    Status(ProxyShellStatusArgs),
}

#[derive(Debug, Args, Default)]
pub struct ProxyShellEnableArgs {
    #[arg(long = "shell", value_enum, help = "Override shell detection.")]
    pub shell: Option<ProxyShellKind>,
}

#[derive(Debug, Args, Default)]
pub struct ProxyShellDisableArgs {
    #[arg(long = "shell", value_enum, help = "Override shell detection.")]
    pub shell: Option<ProxyShellKind>,
}

#[derive(Debug, Args, Default)]
pub struct ProxyShellStatusArgs {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ProxyShellKind {
    Bash,
    Zsh,
    Fish,
}

#[derive(Debug, Args)]
pub struct ProxyDesktopArgs {
    #[command(subcommand)]
    pub action: ProxyDesktopAction,
}

#[derive(Debug, Subcommand)]
pub enum ProxyDesktopAction {
    #[command(about = "Point the Linux desktop proxy settings at xrat endpoints.")]
    Enable(ProxyDesktopEnableArgs),
    #[command(about = "Reset the Linux desktop proxy settings to none.")]
    Disable(ProxyDesktopDisableArgs),
    #[command(about = "Show the current Linux desktop proxy settings.")]
    Status(ProxyDesktopStatusArgs),
}

#[derive(Debug, Args, Default)]
pub struct ProxyDesktopEnableArgs {
    #[arg(long = "desktop", value_enum, help = "Override desktop detection.")]
    pub desktop: Option<ProxyDesktopKind>,
    #[arg(
        long = "pac",
        help = "Configure the desktop to use the PAC URL instead of manual proxies."
    )]
    pub pac: bool,
}

#[derive(Debug, Args, Default)]
pub struct ProxyDesktopDisableArgs {
    #[arg(long = "desktop", value_enum, help = "Override desktop detection.")]
    pub desktop: Option<ProxyDesktopKind>,
}

#[derive(Debug, Args, Default)]
pub struct ProxyDesktopStatusArgs {
    #[arg(long = "desktop", value_enum, help = "Override desktop detection.")]
    pub desktop: Option<ProxyDesktopKind>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum ProxyDesktopKind {
    Gnome,
    Kde,
    Xfce,
}
