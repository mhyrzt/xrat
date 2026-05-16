use clap::Args;

#[derive(Debug, Args)]
#[command(about = "Add a single config URI directly to SQLite.")]
pub struct AddArgs {
    #[arg(
        help = "Config URI to add, e.g. vless://..., vmess://..., ss://..., trojan://..., or hysteria2://..."
    )]
    pub input: String,
}
