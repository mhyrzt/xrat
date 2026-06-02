use crate::app::context::AppContext;
use crate::cli::{GeoIpDownloadArgs, GeoIpUpdateArgs};

use super::download;

pub(crate) async fn run(context: &AppContext, args: &GeoIpUpdateArgs) -> crate::app::Result<()> {
    let download_args = GeoIpDownloadArgs {
        editions: Vec::new(),
        all: true,
        output: args.output.clone(),
        force: true,
        url: args.url.clone(),
        timeout_secs: args.timeout_secs,
        quiet: args.quiet,
    };

    download::run(context, &download_args).await
}
