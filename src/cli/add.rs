use clap::Args;

#[derive(Debug, Args)]
pub struct AddArgs {
    #[arg(
        help = "Single config URI/text to add, such as vless://..., vmess://..., ss://..., or trojan://..."
    )]
    pub input: String,
}
